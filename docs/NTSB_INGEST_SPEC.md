# NTSB Fixed-Wing Accident Pipeline — Ingest & Classification Spec (v0.1)

Owner: Dawson Moore (DWMM Holdings) · Target: Rust workspace built with Claude Code · Date: 2026-10-06

This spec covers phase 1 only: **get every NTSB airplane record from 1982 to present into a local analytical store, with each aircraft classified by type**. The analysis method is phase 2 and is out of scope here, but section 9 lists what this phase must preserve so phase 2 is possible.

Companion files:

| File | Purpose |
|---|---|
| `config/aircraft_types.toml` | The taxonomy: 20 classes → 229 families → 276 variants, with NTSB make/model match rules. Source of truth. |
| `data/reference/aircraft_types.csv` | Flat, human-readable view of the same list (one row per variant). |
| `crates/aircraft-taxonomy/tests/fixtures/taxonomy_golden.csv` | 281 raw make/model/series/homebuilt → expected family/variant cases. Contract tests. |
| `crates/aircraft-taxonomy/` | Reference Rust loader + matcher (`src/lib.rs`). Passes 281/281 golden cases. |

---

## 1. Scope

- **In:** NTSB aviation events dated 1982-01-01 to present, all operation types (Part 91, 121, 135, 137, 125, 129, public use, etc.), accidents **and** the incidents NTSB keeps. Keep `far_part` and `ev_type` as columns; filter in analysis, not at ingest.
- **Aircraft scope:** `aircraft.acft_category = 'AIR'` (airplane). Load every other category too (helicopter, glider, balloon, etc.) into the raw tables; only classify `AIR`. Records with blank/`UNK` category are classified if the make/model matches a family, and are flagged.
- **Out:** pre-1982 data (`PRE1982.zip`, different schema), docket PDFs, exposure data (phase 2).
- **Unit of analysis:** one aircraft involvement = (`ev_id`, `Aircraft_Key`). A midair collision is one event with two aircraft rows. Never count aircraft by counting events.

## 2. Data sources

### 2.1 Primary: NTSB avdata MS Access databases

| File | Coverage | Approx. size (zip) | Notes |
|---|---|---|---|
| `avall.zip` → `avall.mdb` | 2008 → present | ~85 MB | Refreshed periodically. Current coding system (Findings + events_sequence). |
| `Pre2008.zip` → `pre2008.mdb` | 1982 → 2007 | ~150 MB | Static. Legacy coding system (Occurrences + seq_of_events). |
| `PRE1982.zip` | 1962 → 1981 | ~38 MB | Out of scope. Different schema. |

Download page: `https://data.ntsb.gov/avdata` (an older NTSB page also points to `https://app.ntsb.gov/avdata/`). **Do not hard-code file URLs.** Fetch the directory page, extract the `href` for each `.zip` by file name, and download those. The page layout has changed before.

Documentation to download alongside (same directory, names may vary): the MDB data dictionary, `codman.pdf` (coding manual), and MDB release notes. Each `.mdb` also contains the table `eADMSPUB_DataDictionary` and code tables (`ct_*`, `dt_*`). Load them; they decode every coded column.

> Note: this spec was written in a sandbox that could not reach `data.ntsb.gov`, so table and column names below are from NTSB documentation and prior use of the files, not from today's download. Milestone M2 makes the pipeline verify them.

### 2.2 Secondary (later): CAROL

CAROL (`https://carol.ntsb.gov`) exports query results as JSON or CSV, including links to final reports and dockets. Use it in phase 2 to attach report URLs by `NtsbNo`. No documented public API; treat it as a manual export input, not a scraping target.

## 3. Workspace layout

```
ntsb-safety/
  Cargo.toml                  # workspace
  config/aircraft_types.toml  # taxonomy (versioned with the code)
  crates/
    ntsb-fetch/               # download + manifest
    ntsb-mdb/                 # MDB -> Arrow/rows (jetdb; mdbtools fallback)
    ntsb-store/               # DuckDB schema, load, core views
    aircraft-taxonomy/        # loader + matcher (included)
    ntsb-cli/                 # `ntsb` binary
  tests/fixtures/taxonomy_golden.csv
  data/                       # gitignored: raw/, db/ntsb.duckdb, reports/
```

Crates:

| Need | Crate | Notes |
|---|---|---|
| HTTP | `reqwest` (blocking, `rustls-tls`) | |
| Zip | `zip` | |
| Access MDB read | `jetdb` (pure Rust, Jet3/Jet4/ACE, MIT/Apache) | v0.3.x, young. Loads whole tables into memory; fine at these sizes. |
| MDB fallback | `mdbtools` (`mdb-tables`, `mdb-export`) as a subprocess | WSL on Windows. Use if `jetdb` fails on a table. Alternative on Windows: `odbc-api` + Microsoft Access Database Engine (64-bit). |
| Store | `duckdb` (feature `bundled`) | Single file DB; Parquet export for free. |
| Taxonomy | `regex`, `serde`, `toml` | |
| CLI / misc | `clap`, `csv`, `sha2`, `chrono`, `tracing`, `anyhow`, `indicatif` | |

Put the MDB reader behind a trait (`trait MdbSource { fn tables() ; fn rows(table) }`) so the backend can change without touching the loader.

## 4. Pipeline stages

### 4.1 `ntsb fetch`

1. Get the directory page; resolve URLs for `avall.zip`, `Pre2008.zip`, and the doc files.
2. Download to `data/raw/<yyyy-mm-dd>/`. Compute SHA-256.
3. Write `data/raw/manifest.json`: file, URL, bytes, sha256, HTTP `Last-Modified`/`ETag`, fetched_at.
4. Skip the download when ETag/Last-Modified/size match the last manifest entry. `--force` overrides.
5. Unzip next to the zip.

### 4.2 `ntsb schema`

Dump each MDB's catalog (tables, columns, types, row counts) to `data/reports/schema_<source>.json`. Compare with the committed snapshot `config/schema_expected.json`. If a table or column the loader uses is missing or renamed, **fail** and print the difference. New columns are a warning.

### 4.3 `ntsb load`

Load every table from both MDBs into DuckDB, verbatim, as text columns: schema `raw_avall.<table>` and `raw_pre2008.<table>`. Add `_source` and `_loaded_at`. Do not cast at this stage; NTSB dates, times and numerics have mixed formats.

Then build typed `core` tables (SQL in `ntsb-store/sql/*.sql`, run in order):

| Core table | From | Key columns (expected names — verify in M2) |
|---|---|---|
| `core.events` | `events` (both) | `ev_id`, `ntsb_no`, `ev_type` (ACC/INC), `ev_date`, `ev_time`, `ev_state`, `ev_country`, `latitude`/`longitude` (+`dec_latitude`/`dec_longitude`), `apt_name`, `ev_nr_apt_id`, `apt_dist`, `light_cond`, `wx_cond_basic` (VMC/IMC), `sky_ceil_ht`, `vis_sm`, `wind_vel_kts`, `gust_kts`, `ev_highest_injury`, `inj_tot_f/s/m/n`, `mid_air`, `on_ground_collision` |
| `core.aircraft` | `aircraft` | `ev_id`, `Aircraft_Key`, `regis_no`, `acft_make`, `acft_model`, `acft_series`, `acft_category`, `homebuilt`, `num_eng`, `far_part`, `damage`, `acft_fire`, `cert_max_gr_wt`, `fixed_retractable`, `type_fly` (purpose), `oper_cert`, `oper_sched`, `phase_flt_spec`, `acft_year`, `afm_hrs` |
| `core.engines` | `engines` | `ev_id`, `Aircraft_Key`, `eng_no`, `eng_type` (REC/TP/TF/TJ/TS/…), `eng_mfgr`, `eng_model`, `hp_or_lbs` |
| `core.crew` | `Flight_Crew` | `crew_no`, `crew_category`, `crew_age`, certificates/ratings, `med_certf`, `crew_inj_level` |
| `core.flight_time` | `flight_time` | `crew_no`, `flight_type` (TOTL/L90D/L30D/L24H), `flight_craft` (ALL/MAKE/PIC/ACTU/NIGH/…), `flight_hours` |
| `core.injury` | `injury` | per-aircraft, per-person-category injury counts |
| `core.narratives` | `narratives` | `narr_accp` (prelim), `narr_accf` (factual), `narr_cause` (probable cause), `narr_inc` |
| `core.findings` | avall `Findings` | `finding_no`, `finding_code`, `finding_description`, `Cause_Factor` (C/F), `modifier_no` — **2008+ coding** |
| `core.events_sequence` | avall `events_sequence` | `Occurrence_No`, `Occurrence_Code`, `Occurrence_Description`, `phase_no`, `Defining_ev` — **2008+ coding** |
| `core.occurrences_legacy` | pre2008 `Occurrences` | `Occurrence_No`, `Occurrence_Code`, `Phase_of_Flight` — **pre-2008 coding** |
| `core.seq_events_legacy` | pre2008 `seq_of_events` | `seq_event_no`, `group_code`, `Subj_Code`, `Cause_Factor`, `Modifier_Code`, `Person_Code` — **pre-2008 coding** |
| `core.codes` | `eADMSPUB_DataDictionary`, `ct_*`, `dt_*` | decode tables |

Rules:

- **De-duplicate** `ev_id` across sources. Prefer the `avall` row; record both sources in `core.events._sources`. Report the overlap count.
- Parse dates to `DATE`; keep the raw string in `*_raw`. Unparseable values go to `data/reports/parse_errors.csv`, not silent NULLs.
- Keep both cause-coding systems in separate tables. Do **not** crosswalk them in phase 1.
- `ev_id` has two formats: `20080107X00026` (to about 2020) and 14 digits, `20250510200140` (CAROL era, about 2021 on;
  9,879 avall events). Accept both everywhere. The date inside a 14-digit id is the record date and can be a day after
  `ev_date`; always use `ev_date`.
- In CAROL-era records `ev_time` can be UTC while the narrative gives local time. Keep `ev_time` and `ev_tmzn` raw; do not
  derive day/night from `ev_time` alone (use `light_cond`).
- `Findings.Cause_Factor` is blank for every CAROL-era finding (17,388 rows). About half of the 2008–2020 rows also repeat
  the role as a " - C"/" - F" suffix inside `finding_description`. Load the raw text; phase 2 normalises it
  (`causal-chain/src/text.rs`).
- `Events_Sequence.Occurrence_Description` is "<phase> <event>" joined by a space; `phase_no` and `eventsoe_no` code the
  two parts. Load both codes so the split never depends on the text.
- Re-running `load` on the same inputs gives the same DB (drop and rebuild `core`; truncate and reload `raw`).

### 4.4 `ntsb classify`

For every `core.aircraft` row, run the taxonomy matcher and write `core.aircraft_class`:

| Column | Meaning |
|---|---|
| `ev_id`, `Aircraft_Key` | key |
| `family_id`, `variant_id` | taxonomy result (variant may be NULL) |
| `class_id` | resolved class (variant override > family class > fallback) |
| `match_status` | `variant` / `family` / `fallback` / `ambiguous` / `out_of_scope` |
| `match_key` | the normalized string that matched (`model` or `model+series`) |
| `ambiguous_with` | family ids when tied |
| `propulsion_taxonomy`, `propulsion_observed`, `propulsion_conflict` | see 5.3 |
| `taxonomy_version` | SHA-256 of `aircraft_types.toml` |

## 5. Taxonomy matching contract

The header of `aircraft_types.toml` is the normative definition. Summary:

1. **Normalize.** `make_norm`: uppercase, every non-`[A-Z0-9]` char → space, collapse spaces, trim. `model_norm` / `series_norm`: uppercase, delete every non-`[A-Z0-9]` char. `combo = model_norm + series_norm` if series is non-empty and `model_norm` does not already end with it.
2. **Family.** Candidate if (any regex of the family's make groups matches `make_norm` **and** `model_re` matches `model_norm`) **or** (aircraft is amateur-built **and** `homebuilt_model_re` matches). If no candidate, retry with `combo`. Highest `priority` wins (default 100). A tie at the top priority is `ambiguous`: report it, never pick one.
3. **Variant.** First variant in file order whose `model_re` matches `combo`, then `model_norm`. No match → family-only result.
4. **Resolve.** Class defaults ← family `attrs` ← variant `class`/`attrs`.
5. **Fallback.** No family → `[[fallback]]` rules in order (`homebuilt` → `unclassified_experimental`; `eng_type` TF/TJ → `unclassified_jet`; TP → `unclassified_turboprop`; REC & ≥2 engines → `unclassified_mep`; REC & 1 → `unclassified_sep`; else `unknown`).

### 5.1 Why builder names matter

Amateur-built aircraft are often registered with the builder's name as the make ("SMITH JOHN", model "RV-7A"). That is why kit families have `homebuilt_model_re`, which ignores the make. It only applies when the NTSB `homebuilt` flag is set.

### 5.2 Known deliberate ambiguities

- A bare `G500` (Gulfstream) is family-only: it can be a GV-SP-era G500 or a GVII-G500.
- `TEXTRON AVIATION` makes both Cessna and Beech models; model strings separate them, except a Textron-made Cessna 350/400 (Corvalis), which falls to King Air 350.

### 5.3 Propulsion reconciliation

Some airframes change propulsion by STC with no change to the model string (PA-46 JetPROP, turbine DHC-3 Otter, turbine Goose/Mallard, turbine DC-3). After matching, compare the taxonomy's `propulsion` with `core.engines.eng_type` (REC → piston, TP → turboprop, TF/TJ → turbofan). If they disagree, set `propulsion_conflict = true` and use the **observed** propulsion for class-level grouping where that changes the class (for example `sep_retract` + TP → `set`). Report the counts.

## 6. QA reports (`ntsb report <name>`) — the taxonomy improvement loop

| Report | Content | Target |
|---|---|---|
| `coverage` | % of AIR aircraft at each `match_status`, by decade and by `far_part` | ≥ 97 % `variant`+`family` for non-homebuilt; ≥ 85 % for homebuilt |
| `unmatched` | Top 500 (`make_norm`, `model_norm`, `homebuilt`) with count among `fallback` rows | Work down from the top; each fix adds a golden case |
| `ambiguous` | Every tie with the families involved | 0 |
| `variant_gaps` | Families with many family-only matches, with the model strings | Add variants where counts justify it |
| `propulsion_conflicts` | Family × observed eng_type counts | Explain or fix every family > 20 rows |
| `class_counts` | Aircraft involvements, fatal involvements, per class/family/year | Sanity check against known totals |

The taxonomy was written from known NTSB naming conventions, not from today's data. **Expect the first `unmatched` report to show gaps.** The loop is: run `classify` → read `unmatched` → edit `aircraft_types.toml` → add the string to `taxonomy_golden.csv` → `cargo test` → repeat until targets are met.

## 7. CLI

```
ntsb fetch [--force]
ntsb schema [--update-snapshot]
ntsb load
ntsb classify [--taxonomy config/aircraft_types.toml]
ntsb report coverage|unmatched|ambiguous|variant_gaps|propulsion_conflicts|class_counts [--csv out.csv]
ntsb export parquet --out data/parquet/
ntsb run            # fetch → schema → load → classify → report coverage
```

## 8. Tests

- `aircraft-taxonomy`: golden CSV test (all rows must pass); normalization unit tests; a test that every regex compiles and every `class`/`makes` key resolves; a test that no two families tie on any golden row.
- `ntsb-store`: tiny fixture DB (a few events per source incl. a midair, a homebuilt, a pre-2008/2008+ pair) → assert row counts, de-dup, date parsing.
- `ntsb-mdb`: if both backends are available, compare row counts per table between `jetdb` and `mdb-export`.
- Idempotency: run `load` + `classify` twice; table hashes must match.

## 9. Data caveats phase 2 depends on (preserve, don't "fix")

1. **Two cause-coding systems.** Pre-2008 occurrences/sequence-of-events codes and 2008+ findings/occurrence codes differ. A crosswalk is a phase-2 design decision.
2. **Status.** Recent events may be preliminary (no findings, no probable cause). Keep a derived `has_probable_cause` flag (findings rows exist or `narr_cause` is non-empty).
3. **Narratives** are sparse before ~1993.
4. **Foreign events** appear only when NTSB led the investigation (some with partial data). Large-jet data will be thin; most large-jet accidents worldwide are not in this set.
5. **Counts are not rates.** NTSB has no flight hours. Phase 2 needs exposure: FAA *General Aviation and Part 135 Activity Survey* (hours by aircraft type group, by year) and BTS T-100 / Form 41 (departures and hours by aircraft type) for airlines. Keep `class_id` mappable to those groupings.
6. **Incidents** (`ev_type = INC`) are a non-random subset, mostly air carrier. Keep them; label them.

## 10. Milestones for Claude Code

| # | Deliverable | Done when |
|---|---|---|
| M1 | Workspace + `aircraft-taxonomy` crate (included) | `cargo test` passes 281/281 golden rows |
| M2 | `fetch` + `schema` | Both MDBs downloaded; schema snapshot committed; column names in §4.3 corrected to what the files contain |
| M3 | `load` (raw + core) | Row counts per table reported; de-dup overlap reported; parse-error report written |
| M4 | `classify` + all reports | `coverage` report produced |
| M5 | Taxonomy iteration | Coverage targets in §6 met; 0 ambiguous; golden set grown with every fix |
| M6 | `export parquet` + `run` | One command rebuilds everything from scratch |

Phase 2 (next): exposure joins, cause crosswalk, and the analysis method itself.
