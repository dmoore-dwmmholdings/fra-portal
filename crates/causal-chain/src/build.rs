//! Chain construction: turns one Involvement into a causal tree rooted at the outcome node.
//!
//! Every node except the outcome gets exactly ONE "what" (the node it explains), so the result is a tree:
//! outcome <- end <- critical <- mechanism <- act <- latent, with context nodes hanging where rules put them.
//! Edges point what <- why. See ANALYSIS_METHOD_SPEC.md §4.
use crate::dict::{Dict, Source, Tier};
use crate::model::{Involvement, Role, RoleFilter};
use std::collections::{BTreeMap, HashMap};

/// How an edge was made. Lets analyses include/exclude weaker inferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prov {
    /// occurrence order (earlier, deeper-or-equal tier explains later)
    Sequence,
    /// pre-2008 finding linked by NTSB to an occurrence number
    Explicit,
    /// `explains` rule match
    Rule,
    /// no rule matched; attached to the deepest compatible spine node
    Default,
    /// outcome <- top of the spine
    Outcome,
}

#[derive(Clone, Debug)]
pub struct GNode {
    /// index into Dict::nodes
    pub node: usize,
    pub role: Role,
    pub seq: Option<u32>,
    /// came from an occurrence row
    pub spine: bool,
    pub defining: bool,
    pub occ_link: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub what: usize,
    pub why: usize,
    pub prov: Prov,
}

#[derive(Clone, Debug)]
pub struct Graph {
    pub id: String,
    pub attrs: BTreeMap<String, String>,
    pub fatal: bool,
    pub nodes: Vec<GNode>,
    pub edges: Vec<Edge>,
    pub outcome: usize,
    pub unmapped: Vec<(Source, String)>,
    /// spine nodes that had no later node of equal-or-shallower tier (attached by rule instead)
    pub inversions: u32,
}

impl Graph {
    pub fn whys(&self, g: usize) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |e| e.what == g)
    }
    pub fn what_of(&self, g: usize) -> Option<&Edge> {
        self.edges.iter().find(|e| e.why == g)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BuildOpts {
    pub roles: RoleFilter,
}
impl Default for BuildOpts {
    fn default() -> Self {
        Self { roles: RoleFilter::CausesAndFactors }
    }
}

pub fn build(dict: &Dict, inv: &Involvement, opts: &BuildOpts) -> Graph {
    let rank = |g: &GNode| dict.nodes[g.node].tier.rank();
    let tier = |g: &GNode| dict.nodes[g.node].tier;
    let mut nodes: Vec<GNode> = Vec::new();
    let mut unmapped = Vec::new();
    let mut occ_gnode: HashMap<u32, usize> = HashMap::new();

    // 1. Spine from occurrences, in sequence order. Same canonical node twice -> merged.
    let mut occs = inv.occurrences.clone();
    occs.sort_by_key(|o| o.seq);
    for o in &occs {
        match dict.map(o.src, &o.text) {
            None => unmapped.push((o.src, o.text.clone())),
            Some(n) => {
                if let Some(ex) = nodes.iter().position(|x| x.node == n) {
                    occ_gnode.insert(o.seq, ex);
                    nodes[ex].defining |= o.defining;
                    continue;
                }
                nodes.push(GNode { node: n, role: Role::Sequence, seq: Some(o.seq), spine: true, defining: o.defining, occ_link: None });
                occ_gnode.insert(o.seq, nodes.len() - 1);
            }
        }
    }
    let n_spine = nodes.len();

    // 2. Findings (role-filtered). Same canonical node as an existing one -> merged, strongest role kept.
    for f in &inv.findings {
        if !opts.roles.keep(f.role) {
            continue;
        }
        match dict.map(f.src, &f.text) {
            None => unmapped.push((f.src, f.text.clone())),
            Some(n) => {
                if let Some(ex) = nodes.iter().position(|x| x.node == n) {
                    if !nodes[ex].spine && f.role > nodes[ex].role {
                        nodes[ex].role = f.role;
                    }
                    continue;
                }
                nodes.push(GNode { node: n, role: f.role, seq: None, spine: false, defining: false, occ_link: f.occ_link });
            }
        }
    }

    // 3. Outcome node.
    let outcome_dict = dict.idx(inv.injury.node_id()).expect("outcome node");
    nodes.push(GNode { node: outcome_dict, role: Role::Sequence, seq: None, spine: false, defining: false, occ_link: None });
    let outcome = nodes.len() - 1;

    let mut edges: Vec<Edge> = Vec::new();
    let mut placed = vec![false; nodes.len()];
    placed[outcome] = true;

    // 4. Sequence edges: each spine node's "what" = nearest LATER spine node with rank <= its rank.
    let mut tops = Vec::new();
    for i in 0..n_spine {
        let ra = rank(&nodes[i]);
        match (i + 1..n_spine).find(|&j| rank(&nodes[j]) <= ra) {
            Some(j) => {
                edges.push(Edge { what: j, why: i, prov: Prov::Sequence });
                placed[i] = true;
            }
            None => tops.push(i),
        }
    }
    // Primary top = shallowest tier, latest in sequence. Outcome <- primary top.
    let mut inversions = 0;
    let mut pending: Vec<usize> = Vec::new();
    if let Some(&primary) = tops.iter().min_by_key(|&&i| (rank(&nodes[i]), std::cmp::Reverse(nodes[i].seq))) {
        edges.push(Edge { what: outcome, why: primary, prov: Prov::Outcome });
        placed[primary] = true;
        for &t in &tops {
            if t != primary {
                inversions += 1;
                pending.push(t);
            }
        }
    }

    // 5. Attach findings (and stray spine tops) by rank ascending, then dictionary order.
    pending.extend(n_spine..outcome);
    pending.sort_by_key(|&i| (rank(&nodes[i]), nodes[i].node));
    for f in pending {
        let rf = rank(&nodes[f]);
        let is_ctx = tier(&nodes[f]) == Tier::Context;
        let compatible = |c: usize, nodes: &Vec<GNode>, placed: &Vec<bool>| {
            c != f
                && placed[c]
                && if is_ctx { tier(&nodes[c]) != Tier::Context } else { rank(&nodes[c]) <= rf }
                && dict.explains(nodes[f].node, nodes[c].node)
        };
        // deepest tier first; same tier: findings before spine nodes, EARLIER spine node before later
        // (earlier = deeper in the chain), then dictionary order (more proximate node listed first)
        let pick = |cands: Vec<usize>, nodes: &Vec<GNode>| {
            cands.into_iter().max_by_key(|&c| {
                let n = &nodes[c];
                (rank(n), !n.spine, std::cmp::Reverse(n.seq.unwrap_or(0)), std::cmp::Reverse(n.node))
            })
        };

        let mut chosen: Option<(usize, Prov)> = None;
        if let Some(k) = nodes[f].occ_link {
            if let Some(&og) = occ_gnode.get(&k) {
                let group: Vec<usize> = (0..nodes.len())
                    .filter(|&c| (c == og || nodes[c].occ_link == Some(k)) && compatible(c, &nodes, &placed))
                    .collect();
                chosen = Some((pick(group, &nodes).unwrap_or(og), Prov::Explicit));
            }
        }
        if chosen.is_none() {
            let cands: Vec<usize> = (0..nodes.len()).filter(|&c| compatible(c, &nodes, &placed)).collect();
            if let Some(c) = pick(cands, &nodes) {
                chosen = Some((c, Prov::Rule));
            }
        }
        if chosen.is_none() {
            let spine: Vec<usize> = (0..n_spine).filter(|&c| placed[c] && (is_ctx || rank(&nodes[c]) <= rf)).collect();
            let t = spine
                .into_iter()
                .max_by_key(|&c| (rank(&nodes[c]), std::cmp::Reverse(nodes[c].seq)))
                .unwrap_or(outcome);
            chosen = Some((t, Prov::Default));
        }
        let (t, prov) = chosen.unwrap();
        edges.push(Edge { what: t, why: f, prov });
        placed[f] = true;
    }

    Graph {
        id: inv.id.clone(),
        attrs: inv.attrs.clone(),
        fatal: inv.injury == crate::model::Injury::Fatal,
        nodes,
        edges,
        outcome,
        unmapped,
        inversions,
    }
}

/// Render a graph as an indented tree (debugging / reports).
pub fn render(dict: &Dict, g: &Graph) -> String {
    fn walk(dict: &Dict, g: &Graph, n: usize, depth: usize, out: &mut String, prov: Option<Prov>) {
        let gn = &g.nodes[n];
        let role = match gn.role {
            Role::Cause => " [C]",
            Role::Factor => " [F]",
            Role::Finding => " [-]",
            Role::Sequence => "",
        };
        let p = prov.map(|p| format!("  <{p:?}>")).unwrap_or_default();
        out.push_str(&format!("{}{}{}{}\n", "  ".repeat(depth), dict.nodes[gn.node].id, role, p));
        for e in g.whys(n) {
            walk(dict, g, e.why, depth + 1, out, Some(e.prov));
        }
    }
    let mut s = String::new();
    walk(dict, g, g.outcome, 0, &mut s, None);
    s
}
