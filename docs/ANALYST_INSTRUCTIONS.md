# Analyst instructions (batch agents)

You analyze NTSB airplane accident records one at a time and store each analysis in the FRA portal with
`tools/batch/put.py`. The batch loop gives you a batch directory and a list of input files.

Repo: `C:\Coding\random-ideas\flight-report-analysis`. Do not edit, commit or push anything in it.
Method: `docs/ANALYSIS_METHOD_SPEC.md` §1–§4. Node dictionary: `config/causal_nodes.toml` (v0.2).
Aircraft taxonomy: `data/reference/aircraft_types.csv` (use `resolved_class_id` as `class_id`, and `family_id`).
Never write test or placeholder records: every store must be a real, complete analysis.
Do not use the `mcp__fra-portal__*` tools: their cached schema in this session is stale (old ev_id pattern, no
`event`). Never change an ev_id to fit a schema; use it exactly as the input file gives it.

## Each input file

`data/reports/batches/<batch>/<ev_id>_<aircraft_key>.txt` holds one aircraft involvement: header facts, an
`## event` JSON line, the event sequence and findings **already mapped to dictionary nodes**, and NTSB's
narratives (`narr_cause` probable cause, `narr_accf` analysis, `narr_accp` factual).

For each file:

1. Read it in full. The chain comes from `narr_cause` and `narr_accf`; the coded rows support it.
2. Build the narrative chain: a tree rooted at `outcome.<outcome>`, following the ladder in spec §2
   (end → critical → mechanism → act → latent, context where it fits). Use dictionary node ids. When no node
   fits, use `proposed: <tier>.<name>` and say so in `dictionary_feedback`.
3. Roles from `narr_cause`: `"C"` if in the probable cause, `"F"` if "contributing", otherwise `null`.
4. Write the arguments below as JSON to `data/reports/batches/<batch>/<ev_id>_<aircraft_key>.json`, then run
   `python tools/batch/put.py <that file>` from the repo root. Arguments:
   - `ev_id`, `aircraft_key`, `ntsb_no`, `event_date` from the header; `outcome` from the header (this aircraft).
   - `event`: the `## event` JSON line, copied as-is (an object).
   - `aircraft`: `make`, `model`, and the best taxonomy `class_id` / `family_id` (omit those two if no match).
   - `summary`: markdown, at most 120 words: a bold one-line headline; 2–3 sentences of what happened; a
     **Chain:** line (`fatal ← collision ← … ← root why`); a **Lesson:** line for pilots. Facts from the file only.
   - `nodes`: every dictionary node id on the narrative chain (no proposed ones).
   - `analyst`: `"claude-opus-5-5"`; `method_version`: `"wwca-0.2"`.
   - `analysis`, exactly this shape (the viewer reads `narrative_chain.tree`):

```json
{
  "narrative_chain": {
    "provenance": "Narrative",
    "source": "narr_cause + narr_accf",
    "tree": {"node": "outcome.fatal", "whys": [
      {"node": "end.collision", "label": "Collision with trees", "whys": [
        {"node": "crit.stall_spin", "role": "C", "label": "Stall/spin after takeoff", "whys": []}
      ]}
    ]}
  },
  "coded_notes": ["Where the §4 builder's chain from the coded rows would differ, in one line each"],
  "dictionary_feedback": ["Only real problems: wrong or missing node mappings, with the row text"],
  "lessons": ["One or two"]
}
```

   `label` is plain words for that accident ("Door not verified latched"), not the node's generic label.
5. `put.py` prints the server's reply and exits non-zero unless it says `created` or `replaced`. On an error,
   fix the JSON and retry once; otherwise list the ev_id as failed.

If the cause is not determined, the chain ends in `und.not_determined` (or `mech.engine.undetermined` for an
unexplained power loss), and the summary says so.

## When your list is done

Append your dictionary feedback, one line per issue (`<count>\t<row text pattern>\t<current node>\t<proposed>`),
to `data/reports/batches/<batch>/feedback_<group>.tsv`.

Then reply with ONE line only: `group <g>: stored <n>/<m>; failed: <ev_id list or none>`.
