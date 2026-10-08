# Flight Report Analysis (FRA) Portal

A web app where pilots break down NTSB fixed-wing accident data with What–Why Chain Analysis
(`docs/ANALYSIS_METHOD_SPEC.md`). Reports are analyzed one at a time by Claude Code and uploaded
to Firestore through a hosted MCP server; the portal on Firebase Hosting reads them.

```
NTSB avdata ──ntsb fetch──▶ data/raw/*.mdb ──Claude Code──▶ MCP (/mcp function) ──▶ Firestore ──▶ web/
```

```
web/                           Vite + React + TS portal (Firebase Hosting)
functions/                     Cloud Functions: `mcp` — Streamable HTTP MCP server (put/get/list_analysis)
firebase.json, firestore.*     Firebase config; analyses are public-read, written only by the function
.mcp.json                      Claude Code MCP client config (token from $FRA_MCP_TOKEN)
docs/
  NTSB_INGEST_SPEC.md          phase 1: fetch/load NTSB MDBs, classify aircraft (spec for Claude Code)
  ANALYSIS_METHOD_SPEC.md      phase 2: What–Why Chain Analysis method (normative)
config/
  aircraft_types.toml          aircraft taxonomy: 20 classes, 229 families, 276 variants
  causal_nodes.toml            causal node dictionary: tiers, explains rules, NTSB text matchers
crates/
  aircraft-taxonomy/           taxonomy loader + matcher (golden test: 281 cases)
  causal-chain/                chain builder + all WWCA metrics + synthetic demo; dict-map / wwca-qa QA tools
  ntsb-fetch/                  download avdata MDBs + docs, SHA-256 manifest, unzip
  ntsb-cli/                    `ntsb` binary (fetch; schema/load/classify to come)
data/reference/
  aircraft_types.csv           flat, human-readable aircraft type list
examples/
  wwca_demo_output_SYNTHETIC.txt   demo output on MADE-UP data (illustration only)
tools/taxonomy-gen/            Python source that generates aircraft_types.toml/.csv + golden CSV
```

## Quick start (from the repo root)

```
cargo test --release                                   # all tests
cargo run --release --bin ntsb -- fetch                # download NTSB MDBs to data/raw/<date>/ (~250 MB zipped, 1.5 GB unzipped)
cargo run --release --bin wwca-demo                    # demo; reads config/causal_nodes.toml, writes ./wwca_demo_out/
cargo run --release --bin wwca-qa -- <csv dir>         # §10 QA over `jetdb export` CSVs of avall (Events_Sequence, Findings, injury)
```

Raw data is gitignored. Inspect the MDBs with `cargo install jetdb-cli`, then
`jetdb tables data/raw/<date>/avall.mdb` / `jetdb export <mdb> <table>`.

## Web app and MCP

```
cd web && cp .env.example .env.local     # fill from `firebase apps:sdkconfig web`
npm install && npm run dev
cd functions && npm install && npm run build
firebase deploy                          # hosting + functions + firestore rules (Blaze plan, for functions)
```

The MCP endpoint is `https://<project>.web.app/mcp`. It requires `Authorization: Bearer <token>`,
where the token is the `MCP_TOKEN` secret (`firebase functions:secrets:set MCP_TOKEN`). Claude Code
picks it up from `.mcp.json` when `FRA_MCP_TOKEN` is set in your environment.

## Regenerating the aircraft taxonomy

`config/aircraft_types.toml` is generated. Edit `tools/taxonomy-gen/taxonomy_data.py` (families) or
`golden_cases.py` (tests), then:

```
python tools/taxonomy-gen/build.py tax_out
cp tax_out/aircraft_types.toml config/
cp tax_out/aircraft_types.csv data/reference/
cp tax_out/taxonomy_golden.csv crates/aircraft-taxonomy/tests/fixtures/
cargo test -p aircraft-taxonomy
```

## Status

- `ntsb fetch` works against data.ntsb.gov. Both MDBs (JET4) read with `jetdb`: avall 31,621 events,
  Pre2008 63,003. Both files contain the same table set (incl. `Findings`, `Events_Sequence`,
  `Occurrences`, `seq_of_events`), so the spec's per-era table split needs checking in M2.
- Node dictionary v0.2 iterated against all avall (2008+) texts: 0.003 % unmapped, 3.4 % Default-provenance edges.
  Pre-2008 matchers are still untested against real data.
- 112 fatal 2024–25 airplane accidents analyzed and stored in the portal (method wwca-0.1).
- Web app and MCP server are scaffolds; the analysis workflow comes next.
