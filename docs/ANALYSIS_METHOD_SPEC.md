# What–Why Chain Analysis (WWCA) — Method Spec v0.1

Owner: Dawson Moore (DWMM Holdings) · Phase 2 of the NTSB pipeline · Date: 2026-10-06
Reference implementation: `crates/causal-chain/` (Rust, 9 tests passing). Node dictionary: `config/causal_nodes.toml`.

---

## 0. Idea in one paragraph

Every accident is a chain of causes. "What" and "why" are not fixed categories. They are **positions on that chain**. The fatal collision is a *what*, and the forced landing is its *why*. The forced landing is a *what*, and the power loss is its *why*. Then fuel starvation, then the pilot's fuel error. WWCA builds this chain for every aircraft in every NTSB record, using one shared vocabulary for both coding eras. Any node on it can then be picked as the *what*, and the tool shows the percentage breakdown of its *whys*, or the reverse. You choose how many links deep to look, how coarse the buckets are, and which group of aircraft or flights to include. The same chains also give the high-level view: what share of accidents each "what" and each "why" accounts for.

```
outcome.fatal
  end.collision                       ← WHAT the user sees first
    end.forced_landing                ← its WHY … and the next WHAT
      crit.power_loss.total           ← WHY of the forced landing
        mech.fuel.starvation [C]      ← WHY of the power loss
          act.pilot.fuel_mgmt [C]     ← WHY of the starvation (root why)
      env.terrain [F]                 ← context: trees at the landing site
```
(This is the exact tree the reference implementation builds for that case. Test `user_example_builds_expected_chain`.)

## 1. Terms

| Term | Definition |
|---|---|
| **Involvement** | One aircraft in one event: (`ev_id`, `Aircraft_Key`). The unit of counting. |
| **Node** | A canonical concept from `causal_nodes.toml`, e.g. `mech.fuel.starvation`. The id is a dot path. |
| **Tier** | The node's rung on the causal ladder (§2). Fixed per node. |
| **Bucket / granularity g** | The first *g* segments of a node id. g=1 → `mech`, g=2 → `mech.fuel`, g=0 → full id. This is the "bucketing" for high-level views. |
| **Edge** | `what ← why`. Each node except the outcome has exactly one *what*, so each involvement is a **tree** rooted at its outcome. |
| **Depth** | Number of edges between the focal node and an answer node. |
| **Root why** | A leaf of the why-subtree: the deepest cause NTSB coded on that branch. |
| **Role** | NTSB `Cause_Factor`: C (cause), F (factor), blank (finding). Occurrence-derived nodes have role *sequence*. |
| **Provenance** | How an edge was made: Sequence, Explicit, Rule, Default, Outcome (§4). |

## 2. The causal ladder

| Rank | Tier | Answers | Example nodes |
|---|---|---|---|
| 0 | `outcome` | How bad? | fatal, serious, minor, none |
| 1 | `end` | How did the flight end? | collision, forced landing, CFIT, hard landing, runway excursion, gear-up, nose-over, midair |
| 2 | `critical` | What went wrong? | total/partial power loss, LOC-I, stall/spin, VFR into IMC, LOC on ground, system failure |
| 3 | `mechanism` | By what physical/physiological means? | fuel starvation/exhaustion, carb ice, engine internal failure, AOA exceeded, spatial disorientation, performance not attained |
| 3 | `undetermined` | — | NTSB could not determine |
| 4 | `act` | Who did/decided what? | pilot fuel management, airspeed control, directional control, decision-making, maintenance error, ATC |
| 5 | `latent` | What set that person up? | experience, training, impairment, fatigue, operator/organization, oversight |
| 6 | `context` | Under what conditions? | IMC, wind/gusts, density altitude, dark night, terrain, wildlife |

Rules: lower rank = closer to the outcome. A *why* has the same rank as its *what* or a higher one. The only exception is `context`, which can explain any non-context node. The ladder is a guide for bucketing and attachment. Users are never limited to adjacent tiers: any node can be focal.

## 3. Inputs (from phase-1 `core` tables)

| WWCA record | 2008+ (avall) | pre-2008 |
|---|---|---|
| Occurrence (spine) | `events_sequence`: `Occurrence_No` → `seq`; event part of `Occurrence_Description` ("Phase-**Event**") → `text`; `Defining_ev` | `Occurrences`: `Occurrence_No`, decoded occurrence code → `text` |
| Finding | `Findings`: `finding_description` → `text`; `Cause_Factor` → role; no occurrence link | `seq_of_events`: decode subject/modifier/person → `"SUBJECT - MODIFIER - PERSON"`; `Cause_Factor`; **`Occurrence_No` → explicit link** |
| Outcome | Highest injury among this aircraft's occupants (`injury` table). Fall back to `events.ev_highest_injury` for single-aircraft events | same |
| Cohort attributes | `core.aircraft_class` (class/family/variant), year, `far_part`, era, phase of the defining occurrence, `light_cond`, `wx_cond_basic`, PIC total hours bucket, homebuilt | same |

Keep the phase of each spine occurrence as an attribute of that node instance (planned field; not in v0.1 code). Phase is a cohort filter, not a node.

## 4. Chain construction (normative; `src/build.rs`)

**4.1 Map.** Each row maps to the node of the first matcher, in file order, whose `src` equals the row's source and whose regex matches `text`. Rows that match nothing go to `unmapped`. Nothing is dropped silently.

**4.2 Role filter.** Default `CausesAndFactors`. Plain findings are dropped before building. The other options are `CausesOnly` and `All`. Occurrences are always kept.

**4.3 Merge.** If two rows map to the same node in one involvement, they make one node. A finding keeps the strongest role.

**4.4 Sequence edges.** Sort spine nodes by `seq`. Each spine node's *what* is the **nearest later** spine node with rank ≤ its own rank (earlier and deeper explains later). Spine nodes with no such node are *tops*.

**4.5 Outcome edge.** The primary top is the lowest rank, latest `seq`. The edge is `outcome ← primary top`. Other tops are *inversions* (NTSB listed them out of causal order) and are attached in 4.6.

**4.6 Attach findings and inversions** in order of (rank ascending, dictionary file order). Each node *f* gets one *what*:

1. **Explicit** (pre-2008 only): pick from the linked occurrence node plus already-placed findings with the same link. Use the deepest one that *f* `explains`. If none qualifies, attach to the linked occurrence node itself.
2. **Rule**: from all placed nodes *c* where `rank(c) ≤ rank(f)`, or any non-context node if *f* is context, and *f*.`explains` has a prefix of *c*. Pick the **deepest** one.
3. **Default**: the deepest placed spine node with rank ≤ rank(f), else the outcome.

Tie-break for "deepest" is, in order: higher rank; then finding over spine node; then **earlier** spine `seq` (deeper in the chain); then earlier in the dictionary. List more proximate nodes first in a tier. For example, `act.pilot.decision` comes before `act.pilot.wx_planning`, so weather planning can attach under the decision.

**4.7 Guarantees.** The result is a tree. Every non-outcome node has one *what* (test `every_node_has_exactly_one_what`). No cycles. Every edge carries its provenance.

Why a tree and not a general graph? It keeps percentages well defined and paths unique. If a finding plausibly explains two nodes, it is attached to the deeper one, and the shallower one still reaches it through the tree. Analyses at depth `Any` or `Root` therefore see it from both.

## 5. Metrics (normative; `src/metrics.rs`)

**5.1 Focal set.** All nodes in an involvement whose id has the focal prefix (segment-aware: `mech.fuel` matches `mech.fuel.starvation`, not `mech.fuelx`). Nodes inside the focal bucket are passed through. For example, with focal `end`, the forced landing is not reported as a why of the collision. The first why outside the bucket is reported.

**5.2 Answer set.** It is set by `depth` ∈ {Direct, Exactly(k), Root, Any} and `ctx` ∈ {Include, Exclude, Only}. With `exclude_default`, Default-provenance edges are ignored. For *whats*, the outcome is excluded; use fatal rate and lethality for severity.

**5.3 Per-involvement weights.** Map each answer node to its bucket at granularity g. Each bucket gets the maximum role weight of its nodes: sequence 1.0, cause 1.0, factor 0.5, finding 0.25 (configurable in `[weights]`). Attribution within the involvement: `a_e(b) = w_b / Σ w`. An empty answer set becomes the bucket `(none recorded)` with a = 1.

**5.4 The two percentages.** Always show both.

| Metric | Formula | Reads as | Sums to |
|---|---|---|---|
| **Incidence** | `I(b|X) = n_b / N_X` | "In X % of power-loss accidents, fuel exhaustion is a coded why." | > 100 % (multi-cause) |
| **Attribution** | `A(b|X) = Σ_e a_e(b) / N_X` | "Fuel exhaustion carries Y % of the causal weight behind power losses." | exactly 100 % incl. `(none recorded)` |

`A ≤ I` always. Both are tested.

- **CIs:** Wilson 95 % for incidence; percentile bootstrap (B = 400, resample involvements, fixed seed) for attribution.
- **Fatal rate per row:** fatal involvements / n_b.
- **Display rules:** flag `low_n` when N_X < 30 or n_b < 5. Do not publish a row with n_b < 5 (show "<5").

## 6. Analyses

| # | Analysis | Question | Function / CLI |
|---|---|---|---|
| 6.1 | **Why drill-down** | Pick a WHAT → breakdown of its WHYs (direct, k-deep, or root) | `why_breakdown` · `wwca why --what crit.power_loss --depth direct --gran 0` |
| 6.2 | **What drill-up** | Pick a WHY → breakdown of the WHATs it leads to | `what_breakdown` · `wwca what --why act.pilot.decision --depth any` |
| 6.3 | **Tier overview** | High-level "% of WHATs" and "% of WHYs" | `tier_overview` · `wwca tiers --tier critical --gran 2` |
| 6.4 | **Causal flow** | Every what→why link with share_of_what / share_of_why (Sankey across the ladder) | `flow` · `wwca flow --gran 2` |
| 6.5 | **Chain signatures** | Most common full causal paths, with share and fatal rate | `chains` · `wwca chains --gran 2 --ctx exclude --top 50` |
| 6.6 | **Lethality lens** | P(fatal \| node), Wilson CI, lift over the cohort baseline | `lethality` · `wwca lethality --tier critical` |
| 6.7 | **Cohort compare** | Same question for two cohorts. Two-proportion z on incidence, Benjamini–Hochberg q | `compare` · `wwca compare --why-of crit.power_loss --a "class=sep_fixed" --b "class=sep_retract"` |

**Standard high-level report**, produced for every cohort:

- WHATs: tiers `end` and `critical` at g = 0 and 2.
- WHYs: `mechanism`+`undetermined`, `act`, `latent` and `context` at g = 0 and 2.
- Lethality of `critical`.
- The top 25 chain signatures.

**Cohort syntax:** `class=sep_fixed|sep_retract;year>=2008;far_part!=121`.

Reading guidance:

- For depth `Any`, use **incidence**. Attribution there splits weight across a whole path and is not meaningful.
- `(none recorded)` is information, not noise. A high share means NTSB stopped coding at that rung. Report it, don't hide it.
- Selecting the outcome as the focal node, in a fatal-only cohort, gives "how fatal flights ended."

## 7. Era harmonization (mandatory check)

The two coding systems map into one vocabulary, but they do not code the same way. The demo shows the typical problem. A pre-2008 power loss with no cause found is coded as a generic undetermined (`und.not_determined`). The 2008+ equivalent is coded as an engine-specific one (`mech.engine.undetermined`). Each looks like 0 % in the other era.

Rules:

1. Every cross-era result is also run split by era (`compare`, era=2008+ vs era=pre2008).
2. A bucket with era q < 0.05 is published only after one of three steps: it is explained as a real trend; the dictionary is fixed (one node, or an equivalence group, planned as `[[equivalence]]`); or the bucket is reported per era.
3. The dictionary file records every known asymmetry in a comment next to the affected nodes.

## 8. Sensitivity checks (mandatory before publishing a ranking)

Re-run with each of the following:

- Role filter `CausesOnly` and `All`
- `exclude_default = true`
- Factor weight 1.0
- `ctx = Exclude`

If the top-5 order changes, publish the range, not one number.

## 9. Optional narrative layer (phase 2b)

NTSB probable-cause sentences are already why-chains: "*The pilot's improper fuel management, which resulted in a total loss of engine power due to fuel starvation.*" An LLM pass over `narr_cause` and `narr_accf` can extract chains, restricted to node ids from the dictionary. These chains are stored with provenance `Narrative` and kept separate from coded chains. Uses:

1. Fill involvements with no findings (preliminary reports, sparse pre-1993 coding).
2. Check the rule-inferred edges in 2008+ data: report edge-level precision/recall of coded-and-inferred against narrative chains. Lower trust in rules that disagree often.

Never mix narrative edges into coded results without a filter flag.

## 10. QA loop (same pattern as the aircraft taxonomy)

| Check | Target |
|---|---|
| Unmapped rows (top 500 texts by count) | < 2 % of rows; fix the dictionary, add a case to `tests/node_golden.rs` |
| Default-provenance share of finding edges | < 5 %; above that, `explains` rules are missing |
| Inversions | Inspect any involvement with > 1 |
| `(none recorded)` by tier and era | Track; large era gaps → §7 |
| Golden tests | `cargo test` green |

## 11. Limitations to state with every published number

- **Counts are not rates.** Without exposure data (flight hours), this shows how accidents happen, not how risky a type or activity is. Add exposure in phase 3.
- **NTSB coding choices drive results.** Coders' habits, and the 2008 coding change, are part of the signal.
- **2008+ finding-to-occurrence links are inferred by rules.** Provenance makes that visible, and §8 tests how much the answers depend on it.
- **Accidents only.** There is no denominator of safe flights with the same why. Lethality is conditional on an accident having happened.
- **Tree simplification.** Real causation is a graph. The tree keeps every cause; it only picks one parent for each.

## 12. Implementation map (for Claude Code)

| Item | Location |
|---|---|
| Dictionary loader + matchers | `crates/causal-chain/src/dict.rs` |
| Input records | `crates/causal-chain/src/model.rs` |
| Chain builder | `crates/causal-chain/src/build.rs` |
| All metrics, cohort filter, QA, CSV | `crates/causal-chain/src/metrics.rs` |
| Demo on synthetic data | `crates/causal-chain/src/bin/wwca_demo.rs` → `examples/wwca_demo_output_SYNTHETIC.txt` |
| Tests | `crates/causal-chain/tests/chain.rs` (8), `crates/causal-chain/tests/node_golden.rs` (84 text cases) |

Integration steps:

1. **Adapter.** `core.*` (phase 1) → `Vec<Involvement>`. One SQL query per source table, grouped by (`ev_id`, `Aircraft_Key`). Decode pre-2008 codes with `core.codes`.
2. **Persist.** Write `causal.node_instance` (inv_id, node_id, tier, role, seq, spine, phase) and `causal.edge` (inv_id, what, why, prov) to DuckDB, so any UI can query without rebuilding.
3. **CLI** `wwca`: `why`, `what`, `tiers`, `flow`, `chains`, `lethality`, `compare`, `qa`. Output as table, CSV, or JSON (`{title, n_focal, n_fatal, rows:[{bucket, n, incidence, inc_ci, attribution, attr_ci, fatal_rate, low_n}]}`).
4. **Dictionary iteration** on real data until the §10 targets are met.

| Milestone | Done when |
|---|---|
| W1 | Adapter + persist; QA report on full 1982–present data |
| W2 | Dictionary iterated to §10 targets; golden set ≥ 300 cases |
| W3 | CLI with JSON output; standard high-level report per aircraft class |
| W4 | Era-harmonization and sensitivity reports automated |
| W5 (opt.) | Narrative layer + agreement report |
