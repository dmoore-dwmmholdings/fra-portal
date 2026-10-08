"""Pick the next batch of NTSB airplane involvements to analyze and write one input file per involvement.

Works backward in time from --anchor (the newest report analyzed when the loop started): eligible = airplane
(acft_category AIR) with an NTSB probable cause (final report), not yet stored in the portal, not claimed by a
batch in the last CLAIM_HOURS. Reports on or before the anchor go newest first, so the frontier moves back
through the years; reports after the anchor follow once those run out. A gap left by a failed batch is picked
up again on the next run.

usage (repo root):
  python tools/batch/next_batch.py --csv data/raw/2026-10-07/csv --n 120 --groups 8
Writes data/reports/batches/<batch id>/{<ev_id>_<key>.txt, groups.json} and appends to data/reports/claims.tsv.
Prints the batch id, the group lists and how many eligible involvements remain.

The CSVs are `jetdb export <avall.mdb> <table>` output (events, aircraft, Events_Sequence, Findings, injury,
narratives). Rows are pre-mapped to dictionary nodes with target/release/dict-map (cargo build --release).
"""
import argparse
import collections
import csv
import io
import json
import subprocess
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPORTS = ROOT / "data/reports"
CLAIM_HOURS = 6
FIRESTORE = "https://firestore.googleapis.com/v1/projects/fra-portal/databases/(default)/documents/analyses"
PART = {"091": "Part 91", "135": "Part 135", "137": "Part 137", "121": "Part 121", "133": "Part 133", "125": "Part 125",
        "PUBU": "Public use", "091K": "Part 91 subpart K", "129": "Part 129", "103": "Part 103"}
LIGHT = {"DAYL": "daylight", "NITE": "night", "NDRK": "night, dark", "NBRT": "night, bright", "DUSK": "dusk", "DAWN": "dawn"}


def read_csv(path: Path):
    """jetdb writes mostly UTF-8 with stray cp1252 bytes; decode line by line."""
    lines = []
    for raw in path.read_bytes().split(b"\n"):
        try:
            lines.append(raw.decode("utf-8"))
        except UnicodeDecodeError:
            lines.append(raw.decode("cp1252", errors="replace"))
    return csv.DictReader(io.StringIO("\n".join(lines)))


def clean(s: str) -> str:
    return " ".join((s or "").replace("�", "'").split())


def stored_keys() -> set[tuple[str, str]]:
    keys, token = set(), ""
    while True:
        url = f"{FIRESTORE}?pageSize=1000&mask.fieldPaths=ev_id&mask.fieldPaths=aircraft_key" + (f"&pageToken={token}" if token else "")
        with urllib.request.urlopen(url, timeout=60) as r:
            page = json.loads(r.read())
        for d in page.get("documents", []):
            f = d["fields"]
            keys.add((f["ev_id"]["stringValue"], str(f.get("aircraft_key", {}).get("integerValue", "1"))))
        token = page.get("nextPageToken", "")
        if not token:
            return keys


def recent_claims() -> set[tuple[str, str]]:
    path = REPORTS / "claims.tsv"
    if not path.exists():
        return set()
    cutoff = time.time() - CLAIM_HOURS * 3600
    out = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        t, ev, key, _batch = line.split("\t")
        if float(t) >= cutoff:
            out.add((ev, key))
    return out


def dict_map(rows: list[tuple[str, str]]) -> list[str]:
    exe = ROOT / "target/release" / ("dict-map.exe" if sys.platform == "win32" else "dict-map")
    tsv = "".join(f"{src}\t1\t{text.replace(chr(9), ' ')}\n" for src, text in rows)
    out = subprocess.run([str(exe), "-", str(ROOT / "config/causal_nodes.toml")], input=tsv, capture_output=True,
                         text=True, encoding="utf-8", check=True).stdout
    return [line.split("\t")[2] for line in out.splitlines()]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--csv", required=True, type=Path)
    ap.add_argument("--n", type=int, default=120)
    ap.add_argument("--groups", type=int, default=8)
    ap.add_argument("--anchor", default="2025-05-10", help="start date; older reports first, newer ones last")
    a = ap.parse_args()

    ev = {r["ev_id"]: r for r in read_csv(a.csv / "events.csv")}
    aircraft = [r for r in read_csv(a.csv / "aircraft.csv") if r["acft_category"] == "AIR"]
    narr = {}
    for r in read_csv(a.csv / "narratives.csv"):
        narr[(r["ev_id"], r["Aircraft_Key"])] = r
    has_cause = {k for k, r in narr.items() if (r.get("narr_cause") or "").strip()}

    done = stored_keys() | recent_claims()
    eligible = [r for r in aircraft if (r["ev_id"], r["Aircraft_Key"]) in has_cause and (r["ev_id"], r["Aircraft_Key"]) not in done
                and r["ev_id"] in ev]
    date = lambda r: ev[r["ev_id"]]["ev_date"][:10]
    eligible.sort(key=lambda r: (date(r) <= a.anchor, date(r), r["ev_id"], r["Aircraft_Key"]), reverse=True)
    pick = eligible[: a.n]
    if not pick:
        print(json.dumps({"batch": None, "remaining": 0, "note": "avall exhausted: no eligible airplane involvements left"}))
        return
    keys = {(r["ev_id"], r["Aircraft_Key"]) for r in pick}
    evs = {k[0] for k in keys}

    seq, finds, inj = collections.defaultdict(list), collections.defaultdict(list), collections.defaultdict(list)
    for r in read_csv(a.csv / "Events_Sequence.csv"):
        if r["ev_id"] in evs:
            seq[(r["ev_id"], r["Aircraft_Key"])].append(r)
    for r in read_csv(a.csv / "Findings.csv"):
        if r["ev_id"] in evs:
            finds[(r["ev_id"], r["Aircraft_Key"])].append(r)
    for r in read_csv(a.csv / "injury.csv"):
        if r["ev_id"] in evs:
            inj[(r["ev_id"], r["Aircraft_Key"])].append(r)
    n_aircraft = collections.Counter(r["ev_id"] for r in aircraft if r["ev_id"] in evs)

    batch = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = REPORTS / "batches" / batch
    out.mkdir(parents=True)
    names = []
    for r in pick:
        e, k = r["ev_id"], r["Aircraft_Key"]
        E, N = ev[e], narr.get((e, k), {})
        S = sorted(seq[(e, k)], key=lambda x: int(x["Occurrence_No"] or 0))
        F = sorted(finds[(e, k)], key=lambda x: int(x["finding_no"] or 0))
        nodes = dict_map([("occ08", s["Occurrence_Description"]) for s in S] + [("find08", f["finding_description"]) for f in F])
        fatal = sum(int(i["inj_person_count"] or 0) for i in inj[(e, k)] if i["injury_level"] == "FATL")
        levels = {i["injury_level"] for i in inj[(e, k)] if (i["inj_person_count"] or "0") not in ("0", "")}
        outcome = next((o for lvl, o in [("FATL", "fatal"), ("SERS", "serious"), ("MINR", "minor"), ("NONE", "none")] if lvl in levels), "unknown")
        city, st = (E.get("ev_city") or "").strip().title(), (E.get("ev_state") or "").strip()
        part = (r.get("far_part") or "").strip()
        facts = {
            "location": f"{city}, {st}" if city and st else city or st,
            "operation": PART.get(part, part),
            "conditions": ", ".join(x for x in [(E.get("wx_cond_basic") or "").strip(), LIGHT.get((E.get("light_cond") or "").strip(), "")] if x),
            "fatalities": fatal,
            "probable_cause": clean(N.get("narr_cause", "")),
        }
        L = [f"ev_id: {e}", f"aircraft_key: {k}  (aircraft in this event: {n_aircraft[e]})", f"ntsb_no: {r.get('ntsb_no') or E.get('ntsb_no')}",
             f"event_date: {E['ev_date'][:10]}", f"outcome (this aircraft): {outcome}",
             f"aircraft: {(r.get('acft_make') or '').strip()} | {(r.get('acft_model') or '').strip()} | series {(r.get('acft_series') or '').strip()} | homebuilt {r.get('homebuilt')}",
             f"far_part: {part}  type_fly: {r.get('type_fly')}  damage: {r.get('damage')}  fire: {r.get('acft_fire')}",
             f"weather: {(E.get('wx_cond_basic') or '').strip()} light {E.get('light_cond')} ceiling {E.get('sky_ceil_ht')} vis {E.get('vis_sm')} wind {E.get('wind_dir_deg')}@{E.get('wind_vel_kts')} gust {E.get('gust_kts')}",
             "", "## event (copy into put_analysis.event as-is)", json.dumps(facts, ensure_ascii=False),
             "", "## Events_Sequence (Occurrence_No | description | defining | dictionary v0.2 node)"]
        i = 0
        for s in S:
            L.append(f"{s['Occurrence_No']} | {s['Occurrence_Description'].strip()} | {s['Defining_ev']} | {nodes[i]}")
            i += 1
        L.append("\n## Findings (finding_no | Cause_Factor | description | dictionary v0.2 node)")
        for f in F:
            L.append(f"{f['finding_no']} | {f['Cause_Factor'].strip() or '-'} | {f['finding_description'].strip()} | {nodes[i]}")
            i += 1
        for key in ("narr_cause", "narr_accf", "narr_accp"):
            L.append(f"\n## {key}\n{clean(N.get(key, '')) if key == 'narr_cause' else (N.get(key) or '').strip()}")
        name = f"{e}_{k}"
        (out / f"{name}.txt").write_text("\n".join(L), encoding="utf-8")
        names.append(name)

    groups = [names[g:: a.groups] for g in range(a.groups)]
    groups = [g for g in groups if g]
    (out / "groups.json").write_text(json.dumps(groups), encoding="utf-8")
    now = time.time()
    with (REPORTS / "claims.tsv").open("a", encoding="utf-8") as fh:
        for r in pick:
            fh.write(f"{now:.0f}\t{r['ev_id']}\t{r['Aircraft_Key']}\t{batch}\n")
    dates = [ev[r["ev_id"]]["ev_date"][:10] for r in pick]
    print(json.dumps({"batch": batch, "dir": str(out.relative_to(ROOT)), "count": len(pick), "dates": [min(dates), max(dates)],
                      "remaining_after": len(eligible) - len(pick), "stored_or_claimed": len(done), "groups": groups}))


if __name__ == "__main__":
    main()
