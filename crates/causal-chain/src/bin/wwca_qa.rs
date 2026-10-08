//! QA report (spec §10) over avall CSV exports, before the DuckDB adapter exists.
//!
//! Reads `Events_Sequence.csv`, `Findings.csv` and `injury.csv` as written by `jetdb export <avall.mdb> <table>`,
//! builds every (ev_id, Aircraft_Key) involvement and prints the QA counts plus the §10 shares.
//! Outcome = highest injury level with a non-zero count for that aircraft (spec §3).
//!
//! cargo run --release --bin wwca-qa -- <csv dir> [config/causal_nodes.toml] [--trees <ev_id,...>]
use anyhow::{Context, Result};
use causal_chain::metrics;
use causal_chain::{build, split_find08, normalize_occ08, BuildOpts, Dict, FindRec, Injury, Involvement, OccRec, Prov, Role, Source};
use std::collections::BTreeMap;
use std::path::Path;

fn rows(dir: &Path, table: &str) -> Result<csv::StringRecordsIntoIter<std::fs::File>> {
    let p = dir.join(format!("{table}.csv"));
    Ok(csv::ReaderBuilder::new().flexible(true).from_path(&p).with_context(|| format!("open {}", p.display()))?.into_records())
}

type Invs = BTreeMap<(String, String), Involvement>;

fn inv_mut<'a>(invs: &'a mut Invs, ev: &str, ak: &str) -> &'a mut Involvement {
    invs.entry((ev.to_string(), ak.to_string())).or_insert_with(|| Involvement {
        id: format!("{ev}/{ak}"),
        attrs: BTreeMap::new(),
        injury: Injury::None,
        occurrences: vec![],
        findings: vec![],
    })
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = Path::new(args.first().context("usage: wwca-qa <csv dir> [dict] [--trees ids]")?);
    let dict_path = args.get(1).filter(|a| !a.starts_with("--")).map(String::as_str).unwrap_or("config/causal_nodes.toml");
    let trees: Vec<String> = args.iter().position(|a| a == "--trees").and_then(|i| args.get(i + 1)).map(|s| s.split(',').map(String::from).collect()).unwrap_or_default();
    let dict = Dict::from_toml(&std::fs::read_to_string(dict_path)?)?;

    let mut invs = Invs::new();
    // Events_Sequence: ev_id,Aircraft_Key,Occurrence_No,Occurrence_Code,Occurrence_Description,phase_no,eventsoe_no,Defining_ev
    for r in rows(dir, "Events_Sequence")? {
        let r = r?;
        inv_mut(&mut invs, &r[0], &r[1]).occurrences.push(OccRec {
            seq: r[2].trim().parse().unwrap_or(0),
            src: Source::Occ08,
            text: normalize_occ08(&r[4]).to_string(),
            defining: r[7].trim() == "1",
        });
    }
    // Findings: ev_id,Aircraft_Key,finding_no,finding_code,finding_description,...,Cause_Factor (col 10)
    for r in rows(dir, "Findings")? {
        let r = r?;
        let (text, suffix_role) = split_find08(&r[4]);
        if text.is_empty() {
            continue;
        }
        let role = match Role::from_ntsb(&r[10]) {
            Role::Finding => suffix_role.unwrap_or(Role::Finding),
            coded => coded,
        };
        inv_mut(&mut invs, &r[0], &r[1]).findings.push(FindRec { src: Source::Find08, text: text.to_string(), role, occ_link: None });
    }
    // injury: ev_id,Aircraft_Key,inj_person_category,injury_level,inj_person_count
    for r in rows(dir, "injury")? {
        let r = r?;
        if r[4].trim().parse::<u32>().unwrap_or(0) == 0 {
            continue;
        }
        let lvl = match r[3].trim() {
            "FATL" => Injury::Fatal,
            "SERS" => Injury::Serious,
            "MINR" => Injury::Minor,
            _ => continue,
        };
        if let Some(inv) = invs.get_mut(&(r[0].to_string(), r[1].to_string())) {
            let rank = |i: Injury| match i {
                Injury::Fatal => 3,
                Injury::Serious => 2,
                Injury::Minor => 1,
                Injury::None => 0,
            };
            if rank(lvl) > rank(inv.injury) {
                inv.injury = lvl;
            }
        }
    }

    let graphs: Vec<_> = invs.values().map(|i| build(&dict, i, &BuildOpts::default())).collect();
    for g in graphs.iter().filter(|g| trees.iter().any(|t| g.id.starts_with(t.as_str()))) {
        println!("== {}\n{}", g.id, causal_chain::build::render(&dict, g));
    }
    let refs: Vec<_> = graphs.iter().collect();
    let q = metrics::qa(&dict, &refs, 25);
    let rows_total: usize = invs.values().map(|i| i.occurrences.len() + i.findings.len()).sum();
    let finding_edges = graphs.iter().flat_map(|g| g.edges.iter().filter(|e| !g.nodes[e.why].spine)).count();
    let default_edges = q.edges_by_prov.get(&format!("{:?}", Prov::Default)).copied().unwrap_or(0);
    let pct = |a: usize, b: usize| 100.0 * a as f64 / b.max(1) as f64;
    println!("involvements {}", q.involvements);
    println!("unmapped rows {} / {} = {:.2}%  (target < 2%)", q.unmapped_rows, rows_total, pct(q.unmapped_rows, rows_total));
    println!("default-provenance finding edges {} / {} = {:.2}%  (target < 5%)", default_edges, finding_edges, pct(default_edges, finding_edges));
    println!("inversions {}  (involvements with >1: inspect)", q.inversions);
    println!("edges by provenance {:?}", q.edges_by_prov);
    println!("no end node {} ({:.1}%), roles uncoded {} ({:.1}%), spines reordered {}, implied nodes {}",
        q.no_end, pct(q.no_end, q.involvements), q.roles_uncoded, pct(q.roles_uncoded, q.involvements), q.reordered, q.implied_nodes);
    for (t, n) in &q.unmapped_top {
        println!("  unmapped {n:>6}  {t}");
    }
    if args.iter().any(|a| a == "--defaults") {
        // which why -> what pairs fall back to Default: the `explains` rules to add
        let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
        for g in &graphs {
            for e in g.edges.iter().filter(|e| e.prov == Prov::Default) {
                let id = |i: usize| dict.nodes[g.nodes[i].node].id.clone();
                *pairs.entry((id(e.why), id(e.what))).or_default() += 1;
            }
        }
        let mut v: Vec<_> = pairs.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        for ((why, what), n) in v.into_iter().take(40) {
            println!("  default {n:>5}  {why} -> {what}");
        }
    }
    Ok(())
}
