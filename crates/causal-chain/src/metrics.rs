//! WWCA metrics: why/what breakdowns, tier overview, causal flow, chain signatures, lethality, cohort compare.
//! Definitions are normative in ANALYSIS_METHOD_SPEC.md §5–§6.
use crate::build::{Graph, Prov};
use crate::dict::{bucket, has_prefix, Dict, Tier};
use crate::model::Role;
use std::collections::{BTreeMap, HashMap, HashSet};

pub const NONE_RECORDED: &str = "(none recorded)";
pub const NONE_AT_TIER: &str = "(none at this tier)";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    /// immediate whys / whats
    Direct,
    /// exactly k links away
    Exactly(usize),
    /// whys: leaves of the why-subtree ("root causes as coded"); whats: the outcome
    Root,
    /// every node reachable
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctx {
    Include,
    Exclude,
    Only,
}

#[derive(Clone, Debug)]
pub struct Query {
    /// node-id prefix ("crit.power_loss", "mech.fuel", "act.pilot.fuel_mgmt", "" = everything)
    pub focal: String,
    pub depth: Depth,
    /// roll-up granularity for the answer buckets (segments; 0 = full id)
    pub gran: usize,
    pub ctx: Ctx,
    /// ignore edges with provenance Default
    pub exclude_default: bool,
    /// bootstrap replicates for attribution CIs (0 = off)
    pub bootstrap: usize,
    pub seed: u64,
}

impl Query {
    pub fn new(focal: &str) -> Self {
        Self { focal: focal.into(), depth: Depth::Direct, gran: 0, ctx: Ctx::Include, exclude_default: false, bootstrap: 400, seed: 7 }
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub bucket: String,
    /// involvements (with the focal node) that contain this bucket
    pub n: usize,
    /// n / N_focal  (multi-label; rows can sum > 100 %)
    pub incidence: f64,
    pub inc_ci: (f64, f64),
    /// weighted share; rows sum to exactly 100 %
    pub attribution: f64,
    pub attr_ci: (f64, f64),
    /// fatal involvements / n
    pub fatal_rate: f64,
    pub low_n: bool,
}

#[derive(Clone, Debug)]
pub struct Breakdown {
    pub title: String,
    pub n_focal: usize,
    pub n_fatal: usize,
    pub rows: Vec<Row>,
}

pub fn role_weight(dict: &Dict, r: Role) -> f64 {
    let w = &dict.weights;
    match r {
        Role::Sequence => w.sequence,
        Role::Cause => w.cause,
        Role::Factor => w.factor,
        Role::Finding => w.finding,
    }
}

// ------------------------------------------------------------------ traversal
fn usable_edge(e: &crate::build::Edge, q: &Query) -> bool {
    !(q.exclude_default && e.prov == Prov::Default)
}

fn is_ctx(dict: &Dict, g: &Graph, n: usize) -> bool {
    dict.nodes[g.nodes[n].node].tier == Tier::Context
}

/// gnodes matching the focal prefix
pub fn focal_set(dict: &Dict, g: &Graph, focal: &str) -> Vec<usize> {
    (0..g.nodes.len()).filter(|&i| has_prefix(&dict.nodes[g.nodes[i].node].id, focal)).collect()
}

/// Whys of the focal set at the requested depth (spec §5.2).
pub fn collect_whys(dict: &Dict, g: &Graph, focal: &[usize], q: &Query) -> Vec<usize> {
    let fset: HashSet<usize> = focal.iter().copied().collect();
    let skip_ctx = q.ctx == Ctx::Exclude;
    let children = |n: usize| -> Vec<usize> {
        g.whys(n)
            .filter(|e| usable_edge(e, q))
            .map(|e| e.why)
            .filter(|&c| !(skip_ctx && is_ctx(dict, g, c)))
            .collect()
    };
    // BFS levels; nodes inside the focal bucket are passed through without counting a level
    let mut seen: HashSet<usize> = fset.clone();
    let mut frontier: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = focal.to_vec();
    while let Some(n) = stack.pop() {
        for c in children(n) {
            if fset.contains(&c) {
                continue; // nested focal node: its own children are reached from it directly
            }
            if seen.insert(c) {
                frontier.push(c);
            }
        }
    }
    let mut levels: Vec<Vec<usize>> = Vec::new();
    while !frontier.is_empty() {
        levels.push(frontier.clone());
        let mut next = Vec::new();
        for &n in &frontier {
            for c in children(n) {
                if seen.insert(c) {
                    next.push(c);
                }
            }
        }
        frontier = next;
    }
    let mut out: Vec<usize> = match q.depth {
        Depth::Direct => levels.first().cloned().unwrap_or_default(),
        Depth::Exactly(k) => levels.get(k.saturating_sub(1)).cloned().unwrap_or_default(),
        Depth::Any => levels.concat(),
        Depth::Root => levels.concat().into_iter().filter(|&n| children(n).is_empty()).collect(),
    };
    if q.ctx == Ctx::Only {
        out.retain(|&n| is_ctx(dict, g, n));
    }
    out
}

/// Whats (consequences) of the focal set at the requested depth.
pub fn collect_whats(dict: &Dict, g: &Graph, focal: &[usize], q: &Query) -> Vec<usize> {
    let fset: HashSet<usize> = focal.iter().copied().collect();
    let mut levels: Vec<Vec<usize>> = Vec::new();
    let mut seen: HashSet<usize> = fset.clone();
    let mut frontier: Vec<usize> = Vec::new();
    for &f in focal {
        let mut cur = f;
        // climb through nested focal nodes
        while let Some(e) = g.what_of(cur) {
            if !usable_edge(e, q) {
                break;
            }
            if fset.contains(&e.what) {
                cur = e.what;
                continue;
            }
            if seen.insert(e.what) {
                frontier.push(e.what);
            }
            break;
        }
    }
    while !frontier.is_empty() {
        levels.push(frontier.clone());
        let mut next = Vec::new();
        for &n in &frontier {
            if let Some(e) = g.what_of(n) {
                if usable_edge(e, q) && seen.insert(e.what) {
                    next.push(e.what);
                }
            }
        }
        frontier = next;
    }
    let _ = dict;
    match q.depth {
        Depth::Direct => levels.first().cloned().unwrap_or_default(),
        Depth::Exactly(k) => levels.get(k.saturating_sub(1)).cloned().unwrap_or_default(),
        Depth::Any => levels.concat(),
        Depth::Root => vec![g.outcome],
    }
}

// ------------------------------------------------------------------ aggregation
pub struct Contrib {
    pub fatal: bool,
    /// bucket -> weight (already de-duplicated within the involvement)
    pub items: Vec<(String, f64)>,
}

fn contrib_from_nodes(dict: &Dict, g: &Graph, nodes: &[usize], gran: usize, empty_label: &str) -> Contrib {
    let mut m: BTreeMap<String, f64> = BTreeMap::new();
    for &n in nodes {
        let b = bucket(&dict.nodes[g.nodes[n].node].id, gran);
        let w = role_weight(dict, g.nodes[n].role);
        let e = m.entry(b).or_insert(0.0);
        if w > *e {
            *e = w;
        }
    }
    let mut items: Vec<(String, f64)> = m.into_iter().collect();
    if items.is_empty() {
        items.push((empty_label.to_string(), 1.0));
    }
    Contrib { fatal: g.fatal, items }
}

pub fn wilson(k: usize, n: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 0.0);
    }
    let z = 1.959964f64;
    let (k, n) = (k as f64, n as f64);
    let p = k / n;
    let den = 1.0 + z * z / n;
    let c = (p + z * z / (2.0 * n)) / den;
    let h = z * ((p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt()) / den;
    ((c - h).max(0.0), (c + h).min(1.0))
}

struct SplitMix(u64);
impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

pub fn aggregate(title: String, contribs: Vec<Contrib>, bootstrap: usize, seed: u64) -> Breakdown {
    let n_focal = contribs.len();
    let n_fatal = contribs.iter().filter(|c| c.fatal).count();
    // bucket index
    let mut keys: Vec<String> = Vec::new();
    let mut kidx: HashMap<String, usize> = HashMap::new();
    // per involvement: (bucket idx, attribution)
    let mut per: Vec<Vec<(usize, f64)>> = Vec::with_capacity(n_focal);
    for c in &contribs {
        let tot: f64 = c.items.iter().map(|(_, w)| w).sum();
        let mut v = Vec::new();
        for (b, w) in &c.items {
            let i = *kidx.entry(b.clone()).or_insert_with(|| {
                keys.push(b.clone());
                keys.len() - 1
            });
            v.push((i, if tot > 0.0 { w / tot } else { 1.0 / c.items.len() as f64 }));
        }
        per.push(v);
    }
    let k = keys.len();
    let mut cnt = vec![0usize; k];
    let mut fat = vec![0usize; k];
    let mut att = vec![0f64; k];
    for (c, v) in contribs.iter().zip(&per) {
        for &(i, a) in v {
            cnt[i] += 1;
            att[i] += a;
            if c.fatal {
                fat[i] += 1;
            }
        }
    }
    // bootstrap percentile CI for attribution
    let mut ci = vec![(f64::NAN, f64::NAN); k];
    if bootstrap > 0 && n_focal > 0 {
        let mut rng = SplitMix(seed);
        let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(bootstrap); k];
        let mut acc = vec![0f64; k];
        for _ in 0..bootstrap {
            acc.iter_mut().for_each(|x| *x = 0.0);
            for _ in 0..n_focal {
                for &(i, a) in &per[rng.below(n_focal)] {
                    acc[i] += a;
                }
            }
            for i in 0..k {
                samples[i].push(acc[i] / n_focal as f64);
            }
        }
        for i in 0..k {
            let s = &mut samples[i];
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let lo = s[((bootstrap as f64) * 0.025) as usize];
            let hi = s[(((bootstrap as f64) * 0.975) as usize).min(bootstrap - 1)];
            ci[i] = (lo, hi);
        }
    }
    let nf = n_focal.max(1) as f64;
    let mut rows: Vec<Row> = (0..k)
        .map(|i| Row {
            bucket: keys[i].clone(),
            n: cnt[i],
            incidence: cnt[i] as f64 / nf,
            inc_ci: wilson(cnt[i], n_focal),
            attribution: att[i] / nf,
            attr_ci: ci[i],
            fatal_rate: if cnt[i] > 0 { fat[i] as f64 / cnt[i] as f64 } else { 0.0 },
            low_n: n_focal < 30 || cnt[i] < 5,
        })
        .collect();
    rows.sort_by(|a, b| b.attribution.partial_cmp(&a.attribution).unwrap().then(a.bucket.cmp(&b.bucket)));
    Breakdown { title, n_focal, n_fatal, rows }
}

// ------------------------------------------------------------------ public analyses
/// "Select a WHAT, show its WHYs."  (spec §6.1)
pub fn why_breakdown(dict: &Dict, graphs: &[&Graph], q: &Query) -> Breakdown {
    let mut contribs = Vec::new();
    for g in graphs {
        let f = focal_set(dict, g, &q.focal);
        if f.is_empty() {
            continue;
        }
        let w = collect_whys(dict, g, &f, q);
        contribs.push(contrib_from_nodes(dict, g, &w, q.gran, NONE_RECORDED));
    }
    aggregate(format!("WHY of '{}' depth={:?} gran={} ctx={:?}", q.focal, q.depth, q.gran, q.ctx), contribs, q.bootstrap, q.seed)
}

/// "Select a WHY, show the WHATs it leads to."  (spec §6.2)
pub fn what_breakdown(dict: &Dict, graphs: &[&Graph], q: &Query) -> Breakdown {
    let mut contribs = Vec::new();
    for g in graphs {
        let f = focal_set(dict, g, &q.focal);
        if f.is_empty() {
            continue;
        }
        // outcome nodes are excluded: severity is reported through fatal_rate / lethality instead
        let w: Vec<usize> = collect_whats(dict, g, &f, q).into_iter().filter(|&n| n != g.outcome).collect();
        contribs.push(contrib_from_nodes(dict, g, &w, q.gran, NONE_RECORDED));
    }
    aggregate(format!("WHAT from '{}' depth={:?} gran={}", q.focal, q.depth, q.gran), contribs, q.bootstrap, q.seed)
}

/// Global distribution of one tier ("what % of WHATs / WHYs").  (spec §6.3)
pub fn tier_overview(dict: &Dict, graphs: &[&Graph], tiers: &[Tier], gran: usize, bootstrap: usize, seed: u64) -> Breakdown {
    let contribs = graphs
        .iter()
        .map(|g| {
            let nodes: Vec<usize> = (0..g.nodes.len()).filter(|&i| tiers.contains(&dict.nodes[g.nodes[i].node].tier)).collect();
            contrib_from_nodes(dict, g, &nodes, gran, NONE_AT_TIER)
        })
        .collect();
    let names: Vec<&str> = tiers.iter().map(|t| t.name()).collect();
    aggregate(format!("TIER {} gran={}", names.join("+"), gran), contribs, bootstrap, seed)
}

#[derive(Clone, Debug)]
pub struct FlowRow {
    pub what: String,
    pub why: String,
    pub n: usize,
    /// n / involvements containing `what`
    pub share_of_what: f64,
    /// n / involvements containing `why`
    pub share_of_why: f64,
}

/// Every what<-why link, rolled up. Feeds a Sankey of the causal ladder.  (spec §6.4)
pub fn flow(dict: &Dict, graphs: &[&Graph], gran: usize, ctx: Ctx) -> Vec<FlowRow> {
    let mut pair: HashMap<(String, String), usize> = HashMap::new();
    let mut has: HashMap<String, usize> = HashMap::new();
    for g in graphs {
        let mut seen_pair = HashSet::new();
        let mut seen_b = HashSet::new();
        for i in 0..g.nodes.len() {
            seen_b.insert(bucket(&dict.nodes[g.nodes[i].node].id, gran));
        }
        for e in &g.edges {
            let c = is_ctx(dict, g, e.why);
            if (ctx == Ctx::Exclude && c) || (ctx == Ctx::Only && !c) {
                continue;
            }
            let a = bucket(&dict.nodes[g.nodes[e.what].node].id, gran);
            let b = bucket(&dict.nodes[g.nodes[e.why].node].id, gran);
            if a != b && seen_pair.insert((a.clone(), b.clone())) {
                *pair.entry((a, b)).or_default() += 1;
            }
        }
        for b in seen_b {
            *has.entry(b).or_default() += 1;
        }
    }
    let mut rows: Vec<FlowRow> = pair
        .into_iter()
        .map(|((a, b), n)| FlowRow {
            share_of_what: n as f64 / has[&a] as f64,
            share_of_why: n as f64 / has[&b] as f64,
            what: a,
            why: b,
            n,
        })
        .collect();
    rows.sort_by(|x, y| y.n.cmp(&x.n).then(x.what.cmp(&y.what)));
    rows
}

#[derive(Clone, Debug)]
pub struct ChainRow {
    pub signature: String,
    pub n: usize,
    pub share: f64,
    pub fatal_rate: f64,
}

/// Primary causal path from the outcome's direct why down to the deepest why: at each step take the
/// strongest why (role weight, then deeper tier, then earliest). Identical consecutive buckets collapse.
/// The outcome itself is not part of the signature (use fatal_rate).  (spec §6.5)
pub fn primary_path(dict: &Dict, g: &Graph, gran: usize, ctx: Ctx) -> Vec<String> {
    let mut path: Vec<String> = Vec::new();
    let mut cur = g.outcome;
    let mut guard = 0;
    loop {
        if cur != g.outcome {
            let b = bucket(&dict.nodes[g.nodes[cur].node].id, gran);
            if path.last() != Some(&b) {
                path.push(b);
            }
        }
        let next = g
            .whys(cur)
            .map(|e| e.why)
            .filter(|&c| !(ctx == Ctx::Exclude && is_ctx(dict, g, c)))
            .max_by(|&a, &b| {
                let ka = (role_weight(dict, g.nodes[a].role), dict.nodes[g.nodes[a].node].tier.rank());
                let kb = (role_weight(dict, g.nodes[b].role), dict.nodes[g.nodes[b].node].tier.rank());
                ka.partial_cmp(&kb).unwrap().then(b.cmp(&a))
            });
        match next {
            Some(n) if guard < 32 => {
                cur = n;
                guard += 1;
            }
            _ => break,
        }
    }
    path
}

pub fn chains(dict: &Dict, graphs: &[&Graph], gran: usize, ctx: Ctx, top: usize) -> Vec<ChainRow> {
    let mut m: HashMap<String, (usize, usize)> = HashMap::new();
    for g in graphs {
        let p = primary_path(dict, g, gran, ctx);
        let sig = if p.is_empty() { NONE_RECORDED.to_string() } else { p.join(" <- ") };
        let e = m.entry(sig).or_default();
        e.0 += 1;
        if g.fatal {
            e.1 += 1;
        }
    }
    let n = graphs.len().max(1) as f64;
    let mut rows: Vec<ChainRow> = m
        .into_iter()
        .map(|(s, (k, f))| ChainRow { signature: s, n: k, share: k as f64 / n, fatal_rate: f as f64 / k as f64 })
        .collect();
    rows.sort_by(|a, b| b.n.cmp(&a.n).then(a.signature.cmp(&b.signature)));
    rows.truncate(top);
    rows
}

#[derive(Clone, Debug)]
pub struct LethalRow {
    pub bucket: String,
    pub n: usize,
    pub fatal: usize,
    pub rate: f64,
    pub ci: (f64, f64),
    /// rate / cohort baseline fatal rate
    pub lift: f64,
}

/// P(fatal | node bucket present) for every bucket of the given tiers.  (spec §6.6)
pub fn lethality(dict: &Dict, graphs: &[&Graph], tiers: &[Tier], gran: usize) -> Vec<LethalRow> {
    let base = graphs.iter().filter(|g| g.fatal).count() as f64 / graphs.len().max(1) as f64;
    let mut m: HashMap<String, (usize, usize)> = HashMap::new();
    for g in graphs {
        let bs: HashSet<String> = (0..g.nodes.len())
            .filter(|&i| tiers.contains(&dict.nodes[g.nodes[i].node].tier))
            .map(|i| bucket(&dict.nodes[g.nodes[i].node].id, gran))
            .collect();
        for b in bs {
            let e = m.entry(b).or_default();
            e.0 += 1;
            if g.fatal {
                e.1 += 1;
            }
        }
    }
    let mut rows: Vec<LethalRow> = m
        .into_iter()
        .map(|(b, (n, f))| {
            let rate = f as f64 / n as f64;
            LethalRow { bucket: b, n, fatal: f, rate, ci: wilson(f, n), lift: if base > 0.0 { rate / base } else { f64::NAN } }
        })
        .collect();
    rows.sort_by(|a, b| b.n.cmp(&a.n));
    rows
}

#[derive(Clone, Debug)]
pub struct CompareRow {
    pub bucket: String,
    pub inc_a: f64,
    pub inc_b: f64,
    pub diff: f64,
    pub p: f64,
    /// Benjamini–Hochberg adjusted
    pub q: f64,
}

fn norm_cdf(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26
    let t = 1.0 / (1.0 + 0.3275911 * (x.abs() / std::f64::consts::SQRT_2));
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-(x * x) / 2.0).exp();
    if x >= 0.0 { 0.5 * (1.0 + y) } else { 0.5 * (1.0 - y) }
}

/// Compare incidence of each bucket between two breakdowns (two-proportion z-test, BH-FDR).  (spec §6.7)
pub fn compare(a: &Breakdown, b: &Breakdown) -> Vec<CompareRow> {
    let ma: HashMap<&str, &Row> = a.rows.iter().map(|r| (r.bucket.as_str(), r)).collect();
    let mb: HashMap<&str, &Row> = b.rows.iter().map(|r| (r.bucket.as_str(), r)).collect();
    let keys: std::collections::BTreeSet<&str> = ma.keys().chain(mb.keys()).copied().collect();
    let (na, nb) = (a.n_focal as f64, b.n_focal as f64);
    let mut rows: Vec<CompareRow> = keys
        .into_iter()
        .map(|k| {
            let xa = ma.get(k).map(|r| r.n).unwrap_or(0) as f64;
            let xb = mb.get(k).map(|r| r.n).unwrap_or(0) as f64;
            let (pa, pb) = (xa / na.max(1.0), xb / nb.max(1.0));
            let pp = (xa + xb) / (na + nb).max(1.0);
            let se = (pp * (1.0 - pp) * (1.0 / na.max(1.0) + 1.0 / nb.max(1.0))).sqrt();
            let p = if se > 0.0 { 2.0 * (1.0 - norm_cdf(((pa - pb) / se).abs())) } else { 1.0 };
            CompareRow { bucket: k.to_string(), inc_a: pa, inc_b: pb, diff: pa - pb, p, q: p }
        })
        .collect();
    // BH
    let m = rows.len() as f64;
    let mut idx: Vec<usize> = (0..rows.len()).collect();
    idx.sort_by(|&i, &j| rows[i].p.partial_cmp(&rows[j].p).unwrap());
    let mut prev = 1.0f64;
    for (rank, &i) in idx.iter().enumerate().rev() {
        let qv = (rows[i].p * m / (rank as f64 + 1.0)).min(prev).min(1.0);
        rows[i].q = qv;
        prev = qv;
    }
    rows.sort_by(|x, y| x.q.partial_cmp(&y.q).unwrap());
    rows
}

// ------------------------------------------------------------------ cohort filter
/// "class=sep_fixed;year>=2008;far_part!=121". Numeric comparison when both sides parse as numbers.
#[derive(Clone, Debug, Default)]
pub struct Cohort(Vec<(String, String, String)>);

impl Cohort {
    pub fn parse(s: &str) -> Self {
        let mut v = Vec::new();
        for part in s.split(';').map(str::trim).filter(|p| !p.is_empty()) {
            for op in [">=", "<=", "!=", "=", ">", "<"] {
                if let Some(i) = part.find(op) {
                    v.push((part[..i].trim().to_string(), op.to_string(), part[i + op.len()..].trim().to_string()));
                    break;
                }
            }
        }
        Cohort(v)
    }
    pub fn matches(&self, attrs: &BTreeMap<String, String>) -> bool {
        self.0.iter().all(|(k, op, val)| {
            let Some(a) = attrs.get(k) else { return false };
            match (a.parse::<f64>(), val.parse::<f64>()) {
                (Ok(x), Ok(y)) => match op.as_str() {
                    ">=" => x >= y,
                    "<=" => x <= y,
                    ">" => x > y,
                    "<" => x < y,
                    "!=" => x != y,
                    _ => x == y,
                },
                _ => match op.as_str() {
                    "!=" => a != val,
                    "=" => val.split('|').any(|v| v == a),
                    _ => false,
                },
            }
        })
    }
}

// ------------------------------------------------------------------ QA
#[derive(Debug, Default)]
pub struct Qa {
    pub involvements: usize,
    pub unmapped_rows: usize,
    pub unmapped_top: Vec<(String, usize)>,
    pub inversions: usize,
    pub edges_by_prov: BTreeMap<String, usize>,
    pub no_why_below_outcome: usize,
    /// involvements whose chain has no end-tier node (NTSB coded no collision/landing/etc.)
    pub no_end: usize,
    /// involvements with no cause/factor-coded finding (Auto role filter kept all findings)
    pub roles_uncoded: usize,
    /// involvements whose spine was reordered (§4.4)
    pub reordered: usize,
    /// nodes inserted by `implies` rules
    pub implied_nodes: usize,
}

pub fn qa(dict: &Dict, graphs: &[&Graph], top: usize) -> Qa {
    let mut q = Qa { involvements: graphs.len(), ..Default::default() };
    let mut um: HashMap<String, usize> = HashMap::new();
    for g in graphs {
        q.unmapped_rows += g.unmapped.len();
        for (s, t) in &g.unmapped {
            *um.entry(format!("{s:?}|{t}")).or_default() += 1;
        }
        q.inversions += g.inversions as usize;
        for e in &g.edges {
            *q.edges_by_prov.entry(format!("{:?}", e.prov)).or_default() += 1;
        }
        if g.whys(g.outcome).next().is_none() {
            q.no_why_below_outcome += 1;
        }
        q.no_end += usize::from(!g.has_end(dict));
        q.roles_uncoded += usize::from(!g.roles_coded);
        q.reordered += usize::from(g.reordered > 0);
        q.implied_nodes += g.nodes.iter().filter(|n| n.implied).count();
    }
    let mut v: Vec<(String, usize)> = um.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v.truncate(top);
    q.unmapped_top = v;
    q
}

// ------------------------------------------------------------------ output
pub fn write_breakdown_csv(path: &std::path::Path, b: &Breakdown) -> anyhow::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(["bucket", "n", "incidence", "inc_lo", "inc_hi", "attribution", "attr_lo", "attr_hi", "fatal_rate", "low_n", "n_focal"])?;
    for r in &b.rows {
        w.write_record([
            r.bucket.clone(),
            r.n.to_string(),
            format!("{:.4}", r.incidence),
            format!("{:.4}", r.inc_ci.0),
            format!("{:.4}", r.inc_ci.1),
            format!("{:.4}", r.attribution),
            format!("{:.4}", r.attr_ci.0),
            format!("{:.4}", r.attr_ci.1),
            format!("{:.4}", r.fatal_rate),
            r.low_n.to_string(),
            b.n_focal.to_string(),
        ])?;
    }
    w.flush()?;
    Ok(())
}

pub fn print_breakdown(b: &Breakdown, max_rows: usize) -> String {
    let mut s = format!("{}\n  N = {}  (fatal {})\n", b.title, b.n_focal, b.n_fatal);
    s.push_str(&format!("  {:<34} {:>6} {:>18} {:>18} {:>7}\n", "bucket", "n", "incidence [95% CI]", "attribution [CI]", "fatal%"));
    for r in b.rows.iter().take(max_rows) {
        s.push_str(&format!(
            "  {:<34} {:>6} {:>5.1}% [{:>4.1},{:>4.1}] {:>5.1}% [{:>4.1},{:>4.1}] {:>6.1}{}\n",
            r.bucket,
            r.n,
            100.0 * r.incidence,
            100.0 * r.inc_ci.0,
            100.0 * r.inc_ci.1,
            100.0 * r.attribution,
            100.0 * r.attr_ci.0,
            100.0 * r.attr_ci.1,
            100.0 * r.fatal_rate,
            if r.low_n { "  *low n" } else { "" }
        ));
    }
    s
}
