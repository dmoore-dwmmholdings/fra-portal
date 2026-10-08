//! Chain construction: turns one Involvement into a causal tree rooted at the outcome node.
//!
//! Every node except the outcome gets exactly ONE "what" (the node it explains), so the result is a tree:
//! outcome <- end <- critical <- mechanism <- act <- latent, with context nodes hanging where rules put them.
//! Severity attributes (post-impact fire, evacuation) are recorded on the graph, not placed in the tree.
//! Edges point what <- why. See ANALYSIS_METHOD_SPEC.md §4.
use crate::dict::{has_prefix, Dict, Source, Tier};
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
    /// NTSB Occurrence_No (the first row, when two rows merged into one node)
    pub seq: Option<u32>,
    /// position on the spine after causal reordering (§4.4); None for findings
    pub pos: Option<u32>,
    /// came from an occurrence row
    pub spine: bool,
    pub defining: bool,
    pub occ_link: Option<u32>,
    /// inserted by a dictionary `implies` rule, not coded by NTSB
    pub implied: bool,
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
    /// severity attributes coded as occurrences (post-impact fire, evacuation): Dict node indices
    pub attributes: Vec<usize>,
    /// any finding carried an NTSB cause/factor role (false for CAROL-era records)
    pub roles_coded: bool,
    /// the role filter actually applied (`Auto` resolved)
    pub roles: RoleFilter,
    /// spine nodes moved by causal reordering (§4.4)
    pub reordered: u32,
}

impl Graph {
    pub fn whys(&self, g: usize) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |e| e.what == g)
    }
    pub fn what_of(&self, g: usize) -> Option<&Edge> {
        self.edges.iter().find(|e| e.why == g)
    }
    /// Does the chain record how the flight ended (any end-tier node)?
    pub fn has_end(&self, dict: &Dict) -> bool {
        self.nodes.iter().any(|n| dict.nodes[n.node].tier == Tier::End)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BuildOpts {
    pub roles: RoleFilter,
}
impl Default for BuildOpts {
    fn default() -> Self {
        Self { roles: RoleFilter::Auto }
    }
}

fn gnode(node: usize, role: Role) -> GNode {
    GNode { node, role, seq: None, pos: None, spine: false, defining: false, occ_link: None, implied: false }
}

/// Order spine nodes so that, within one tier, a node that `explains` another comes before it (it is deeper in
/// the chain). NTSB sometimes lists "Loss of control in flight" before the stall or power loss that caused it, and
/// §4.4 would otherwise make the cause the effect's "what". Unconstrained nodes keep NTSB order; a cycle falls back
/// to NTSB order. Returns the new order as indices into `spine`.
fn causal_order(dict: &Dict, spine: &[GNode]) -> Vec<usize> {
    let n = spine.len();
    let rank = |i: usize| dict.nodes[spine[i].node].tier.rank();
    let before = |a: usize, b: usize| {
        rank(a) == rank(b) && dict.explains(spine[a].node, spine[b].node) && !dict.explains(spine[b].node, spine[a].node)
    };
    let mut done = vec![false; n];
    let mut order = Vec::with_capacity(n);
    while order.len() < n {
        let free = (0..n).find(|&b| !done[b] && !(0..n).any(|a| !done[a] && a != b && before(a, b)));
        let next = free.unwrap_or_else(|| (0..n).find(|&b| !done[b]).unwrap());
        done[next] = true;
        order.push(next);
    }
    order
}

pub fn build(dict: &Dict, inv: &Involvement, opts: &BuildOpts) -> Graph {
    let rank = |g: &GNode| dict.nodes[g.node].tier.rank();
    let tier = |g: &GNode| dict.nodes[g.node].tier;
    let roles = opts.roles.resolve(inv);
    let mut unmapped = Vec::new();
    let mut attributes: Vec<usize> = Vec::new();

    // 1. Spine from occurrences, in sequence order. Same canonical node twice -> merged (first seq kept, defining
    //    OR-ed). Attribute nodes are recorded on the graph instead of the spine.
    let mut occs = inv.occurrences.clone();
    occs.sort_by_key(|o| o.seq);
    let mut spine: Vec<GNode> = Vec::new();
    let mut occ_spine: HashMap<u32, usize> = HashMap::new();
    for o in &occs {
        match dict.map(o.src, &o.text) {
            None => unmapped.push((o.src, o.text.clone())),
            Some(n) if dict.nodes[n].attribute => {
                if !attributes.contains(&n) {
                    attributes.push(n);
                }
            }
            Some(n) => {
                if let Some(ex) = spine.iter().position(|x| x.node == n) {
                    occ_spine.insert(o.seq, ex);
                    spine[ex].defining |= o.defining;
                    continue;
                }
                spine.push(GNode { seq: Some(o.seq), spine: true, defining: o.defining, ..gnode(n, Role::Sequence) });
                occ_spine.insert(o.seq, spine.len() - 1);
            }
        }
    }

    // 2. Findings (role-filtered). Same canonical node as an existing one -> merged, strongest role kept.
    let mut finds: Vec<GNode> = Vec::new();
    for f in &inv.findings {
        if !roles.keep(f.role) {
            continue;
        }
        match dict.map(f.src, &f.text) {
            None => unmapped.push((f.src, f.text.clone())),
            Some(n) if dict.nodes[n].attribute => {
                if !attributes.contains(&n) {
                    attributes.push(n);
                }
            }
            Some(n) => {
                if spine.iter().any(|x| x.node == n) {
                    continue;
                }
                if let Some(ex) = finds.iter_mut().find(|x| x.node == n) {
                    ex.role = ex.role.max(f.role);
                    continue;
                }
                finds.push(GNode { occ_link: f.occ_link, ..gnode(n, f.role) });
            }
        }
    }

    // 3. Implied nodes: e.g. fuel starvation with no power-loss node of any kind -> insert crit.power_loss.total,
    //    right after its source on the spine, or as a finding with the source's role.
    let present = |imp: usize, spine: &[GNode], finds: &[GNode]| {
        let group: String = dict.nodes[imp].id.split('.').take(2).collect::<Vec<_>>().join(".");
        spine.iter().chain(finds).any(|x| has_prefix(&dict.nodes[x.node].id, &group))
    };
    let mut i = 0;
    while i < spine.len() {
        if let Some(imp) = dict.nodes[spine[i].node].implies {
            if !present(imp, &spine, &finds) {
                let src = spine[i].clone();
                spine.insert(i + 1, GNode { node: imp, implied: true, defining: false, ..src });
                occ_spine.values_mut().filter(|v| **v > i).for_each(|v| *v += 1);
            }
        }
        i += 1;
    }
    for k in 0..finds.len() {
        if let Some(imp) = dict.nodes[finds[k].node].implies {
            if !present(imp, &spine, &finds) {
                finds.push(GNode { implied: true, occ_link: finds[k].occ_link, ..gnode(imp, finds[k].role) });
            }
        }
    }

    // 4. Causal reordering of the spine (§4.4), then positions.
    let order = causal_order(dict, &spine);
    let reordered = order.iter().enumerate().filter(|&(p, &o)| p != o).count() as u32;
    let mut new_index = vec![0; spine.len()];
    for (p, &o) in order.iter().enumerate() {
        new_index[o] = p;
    }
    let mut nodes: Vec<GNode> = order.iter().map(|&o| spine[o].clone()).collect();
    for (p, n) in nodes.iter_mut().enumerate() {
        n.pos = Some(p as u32);
    }
    let occ_gnode: HashMap<u32, usize> = occ_spine.into_iter().map(|(k, v)| (k, new_index[v])).collect();
    let n_spine = nodes.len();
    nodes.extend(finds);

    // 5. Outcome node.
    let outcome_dict = dict.idx(inv.injury.node_id()).expect("outcome node");
    nodes.push(gnode(outcome_dict, Role::Sequence));
    let outcome = nodes.len() - 1;

    let mut edges: Vec<Edge> = Vec::new();
    let mut placed = vec![false; nodes.len()];
    placed[outcome] = true;

    // 6. Sequence edges: each spine node's "what" = nearest LATER spine node with rank <= its rank.
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
    // Primary top = shallowest tier, latest on the spine. Outcome <- primary top.
    let mut inversions = 0;
    let mut pending: Vec<usize> = Vec::new();
    if let Some(&primary) = tops.iter().min_by_key(|&&i| (rank(&nodes[i]), std::cmp::Reverse(nodes[i].pos))) {
        edges.push(Edge { what: outcome, why: primary, prov: Prov::Outcome });
        placed[primary] = true;
        for &t in &tops {
            if t != primary {
                inversions += 1;
                pending.push(t);
            }
        }
    }

    // 7. Attach findings (and stray spine tops) by rank ascending, then dictionary order.
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
                (rank(n), !n.spine, std::cmp::Reverse(n.pos.unwrap_or(0)), std::cmp::Reverse(n.node))
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
                .max_by_key(|&c| (rank(&nodes[c]), std::cmp::Reverse(nodes[c].pos)))
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
        attributes,
        roles_coded: inv.roles_coded(),
        roles,
        reordered,
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
        let imp = if gn.implied { " (implied)" } else { "" };
        let p = prov.map(|p| format!("  <{p:?}>")).unwrap_or_default();
        out.push_str(&format!("{}{}{}{}{}\n", "  ".repeat(depth), dict.nodes[gn.node].id, role, imp, p));
        for e in g.whys(n) {
            walk(dict, g, e.why, depth + 1, out, Some(e.prov));
        }
    }
    let mut s = String::new();
    walk(dict, g, g.outcome, 0, &mut s, None);
    for &a in &g.attributes {
        s.push_str(&format!("[attribute] {}\n", dict.nodes[a].id));
    }
    s
}
