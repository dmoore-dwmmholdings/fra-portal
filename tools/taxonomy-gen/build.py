"""Generate aircraft_types.toml / .csv / golden csv and run a reference matcher over the golden cases."""
import csv, re, sys, os, datetime
sys.path.insert(0, os.path.dirname(__file__))
from taxonomy_data import CLASSES, FALLBACKS, MAKES, FAMILIES
from golden_cases import CASES

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
CLASS_IDS = {c[0] for c in CLASSES}
CLASS_DEFAULTS = {c[0]: c[3] for c in CLASSES}

# ---------------------------------------------------------------- validation
BAD = re.compile(r"\(\?[=!<]|\\[1-9]")
errs = []
def chk(rx, where):
    if BAD.search(rx): errs.append(f"{where}: lookaround/backref not allowed in Rust regex: {rx}")
    try: re.compile(rx)
    except re.error as e: errs.append(f"{where}: {e}")
for k, lst in MAKES.items():
    for rx in lst: chk(rx, f"make {k}")
ids = set()
for f in FAMILIES:
    if f["id"] in ids: errs.append(f"dup family {f['id']}")
    ids.add(f["id"])
    if f["cls"] not in CLASS_IDS: errs.append(f"{f['id']}: bad class {f['cls']}")
    for m in f["makes"]:
        if m not in MAKES: errs.append(f"{f['id']}: unknown make key {m}")
    chk(f["model_re"], f["id"])
    if f["homebuilt_model_re"]: chk(f["homebuilt_model_re"], f["id"] + " homebuilt")
    vids = set()
    for v in f["variants"]:
        if v["id"] in vids: errs.append(f"{f['id']}: dup variant {v['id']}")
        vids.add(v["id"])
        if v["cls"] and v["cls"] not in CLASS_IDS: errs.append(f"{f['id']}/{v['id']}: bad class")
        chk(v["model_re"], f"{f['id']}/{v['id']}")
if errs:
    print("\n".join(errs)); sys.exit(1)

# ---------------------------------------------------------------- reference matcher
def norm_make(s): return re.sub(r" +", " ", re.sub(r"[^A-Z0-9]", " ", (s or "").upper())).strip()
def norm_model(s): return re.sub(r"[^A-Z0-9]", "", (s or "").upper())
MAKE_RX = {k: [re.compile(r) for r in v] for k, v in MAKES.items()}
for f in FAMILIES:
    f["_rx"] = re.compile(f["model_re"])
    f["_hrx"] = re.compile(f["homebuilt_model_re"]) if f["homebuilt_model_re"] else None
    for v in f["variants"]: v["_rx"] = re.compile(v["model_re"])

def candidates(make, model, homebuilt):
    out = []
    for f in FAMILIES:
        make_ok = any(rx.search(make) for k in f["makes"] for rx in MAKE_RX[k])
        if make_ok and f["_rx"].search(model):
            out.append(f)
        elif homebuilt and f["_hrx"] is not None and f["_hrx"].search(model):
            out.append(f)
    return out

def match(make_raw, model_raw, series_raw, homebuilt):
    make, model, series = norm_make(make_raw), norm_model(model_raw), norm_model(series_raw)
    combo = model + series if series and not model.endswith(series) else model
    keys = [model] + ([combo] if combo != model else [])
    for key in keys:
        c = candidates(make, key, homebuilt)
        if c:
            top = max(f["priority"] for f in c)
            best = [f for f in c if f["priority"] == top]
            if len(best) > 1:
                return ("AMBIGUOUS:" + ",".join(f["id"] for f in best), None, c)
            fam = best[0]
            vkeys = ([combo] if combo != model else []) + [model]
            for vk in vkeys:
                for v in fam["variants"]:
                    if v["_rx"].search(vk):
                        return (fam["id"], v["id"], c)
            return (fam["id"], "", c)
    return (None, None, [])

fails = 0
golden_rows = []
for mk, md, sr, hb, ef, ev in CASES:
    fam, var, c = match(mk, md, sr, hb)
    ok = (fam == ef) and (ef is None or var == ev)
    golden_rows.append(dict(acft_make=mk, acft_model=md, acft_series=sr, homebuilt="Y" if hb else "N",
                            expected_family=ef or "", expected_variant=ev or ""))
    if not ok:
        fails += 1
        print(f"FAIL {mk!r} {md!r} {sr!r} hb={hb}: got {fam}/{var} expected {ef}/{ev}  cands={[x['id']+':'+str(x['priority']) for x in c]}")
print(f"golden: {len(CASES)-fails}/{len(CASES)} pass")

# ---------------------------------------------------------------- emit TOML
def tq(s):  # TOML literal string (regex-safe); fall back to basic string if it contains a quote
    if "'" not in s: return "'" + s + "'"
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'
def tv(v):
    if isinstance(v, bool): return "true" if v else "false"
    if isinstance(v, (int, float)): return str(v)
    if isinstance(v, list): return "[" + ", ".join(tv(x) for x in v) + "]"
    return tq(str(v)) if any(ch in str(v) for ch in "\\^$|") else '"' + str(v) + '"'
def inline(d): return "{ " + ", ".join(f"{k} = {tv(v)}" for k, v in d.items()) + " }"

L = []
L += ["# Fixed-wing aircraft taxonomy for NTSB accident analysis",
      f"# Generated {datetime.date.today().isoformat()} - {len(FAMILIES)} families, "
      f"{sum(len(f['variants']) for f in FAMILIES)} variants, {len(CLASSES)} classes.",
      "#",
      "# Hierarchy: class -> family (airframe lineage) -> variant (specific model).",
      "# A variant may override the family's class and attrs (e.g. 172RG -> sep_retract).",
      "#",
      "# MATCHING CONTRACT (see NTSB_INGEST_SPEC.md section 5):",
      "#   make_norm  = uppercase; every char not [A-Z0-9] -> space; collapse spaces; trim",
      "#   model_norm = uppercase; delete every char not [A-Z0-9]       ('PA-28-161' -> 'PA28161')",
      "#   series_norm likewise; combo = model_norm + series_norm when series is non-empty",
      "#   1. Family candidates: (any make regex in family.makes matches make_norm AND model_re matches model_norm)",
      "#      OR (aircraft is amateur-built AND homebuilt_model_re matches model_norm).",
      "#      If none, retry with combo.  Highest priority wins; ties at top priority = AMBIGUOUS (report, don't guess).",
      "#   2. Variant: first variant (file order) whose model_re matches combo, else model_norm. No match = family-only.",
      "#   3. Class/attrs resolve: class defaults <- family attrs <- variant overrides.",
      "#   4. No family -> [[fallback]] rules in order, using NTSB homebuilt / engines.eng_type / num_eng.",
      "# All regexes are Rust `regex` crate compatible (no lookaround/backrefs); use Regex::is_match (unanchored unless ^/$).",
      "",
      "schema_version = 1",
      'scope = "NTSB aircraft.acft_category = AIR (airplanes), events 1982-present"',
      "", "[makes]"]
for k, v in MAKES.items():
    L.append(f"{k} = [{', '.join(tq(r) for r in v)}]")
L.append("")
for cid, name, desc, d in CLASSES:
    L += ["[[class]]", f'id = "{cid}"', f'name = "{name}"', f'description = "{desc}"', f"defaults = {inline(d)}", ""]
for fid, name, rule in FALLBACKS:
    L += ["[[fallback]]", f'id = "{fid}"', f'name = "{name}"', f"when = {inline(rule)}", ""]
for f in FAMILIES:
    L += ["[[family]]", f'id = "{f["id"]}"', f'name = "{f["name"]}"', f'class = "{f["cls"]}"',
          f"makes = [{', '.join(chr(34)+m+chr(34) for m in f['makes'])}]", f"model_re = {tq(f['model_re'])}"]
    if f["homebuilt_model_re"]: L.append(f"homebuilt_model_re = {tq(f['homebuilt_model_re'])}")
    if f["priority"] != 100: L.append(f"priority = {f['priority']}")
    if f["attrs"]: L.append(f"attrs = {inline(f['attrs'])}")
    if f["notes"]: L.append(f'notes = "{f["notes"]}"')
    for v in f["variants"]:
        L += ["", "  [[family.variant]]", f'  id = "{v["id"]}"', f'  name = "{v["name"]}"', f"  model_re = {tq(v['model_re'])}"]
        if v["cls"]: L.append(f'  class = "{v["cls"]}"')
        if v["attrs"]: L.append(f"  attrs = {inline(v['attrs'])}")
    L.append("")
open(os.path.join(OUT, "aircraft_types.toml"), "w").write("\n".join(L))

# ---------------------------------------------------------------- emit flat CSV (human-readable list)
cname = {c[0]: c[1] for c in CLASSES}
order = [c[0] for c in CLASSES]
with open(os.path.join(OUT, "aircraft_types.csv"), "w", newline="") as fh:
    w = csv.writer(fh)
    w.writerow(["class_id", "class_name", "family_id", "family_name", "variant_id", "variant_name",
                "resolved_class_id", "engines", "propulsion", "gear", "gear_config", "wing"])
    for f in sorted(FAMILIES, key=lambda f: (order.index(f["cls"]), f["name"])):
        base = dict(CLASS_DEFAULTS[f["cls"]]); base.update(f["attrs"])
        rows = f["variants"] or [None]
        for v in rows:
            a = dict(base); rc = f["cls"]
            if v:
                if v["cls"]: rc = v["cls"]; a = dict(CLASS_DEFAULTS[rc]); a.update(f["attrs"])
                a.update(v["attrs"])
            w.writerow([f["cls"], cname[f["cls"]], f["id"], f["name"], v["id"] if v else "", v["name"] if v else "",
                        rc, a.get("engines", ""), a.get("propulsion", ""), a.get("gear", ""), a.get("gear_config", ""), a.get("wing", "")])
with open(os.path.join(OUT, "taxonomy_golden.csv"), "w", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=list(golden_rows[0].keys())); w.writeheader(); w.writerows(golden_rows)
print("families", len(FAMILIES), "variants", sum(len(f["variants"]) for f in FAMILIES))
sys.exit(1 if fails else 0)
