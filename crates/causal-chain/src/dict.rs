//! Causal node dictionary: canonical nodes, tiers, `explains` rules and source-text matchers.
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

/// Causal tier. Rank 0 = most "what" (outcome), higher rank = deeper "why".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Outcome,
    End,
    Critical,
    Mechanism,
    Undetermined,
    Act,
    Latent,
    Context,
}

impl Tier {
    pub fn rank(self) -> u8 {
        match self {
            Tier::Outcome => 0,
            Tier::End => 1,
            Tier::Critical => 2,
            Tier::Mechanism | Tier::Undetermined => 3,
            Tier::Act => 4,
            Tier::Latent => 5,
            Tier::Context => 6,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Tier::Outcome => "outcome",
            Tier::End => "end",
            Tier::Critical => "critical",
            Tier::Mechanism => "mechanism",
            Tier::Undetermined => "undetermined",
            Tier::Act => "act",
            Tier::Latent => "latent",
            Tier::Context => "context",
        }
    }
}

/// Where a raw text row came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// avall events_sequence ("Phase-Event" description), 2008+
    Occ08,
    /// avall Findings.finding_description, 2008+
    Find08,
    /// pre2008 Occurrences, decoded
    OccPre08,
    /// pre2008 seq_of_events, decoded "SUBJECT - MODIFIER - PERSON"
    FindPre08,
    /// optional narrative extraction (spec §9)
    Narrative,
}

#[derive(Deserialize)]
struct RawMatch {
    src: Source,
    re: String,
    /// higher wins over file order (default 0)
    #[serde(default)]
    prio: i32,
}
#[derive(Deserialize)]
struct RawNode {
    id: String,
    label: String,
    tier: Tier,
    #[serde(default)]
    explains: Vec<String>,
    #[serde(default, rename = "match")]
    matchers: Vec<RawMatch>,
    #[serde(default)]
    attribute: bool,
    #[serde(default)]
    implies: Option<String>,
}
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Weights {
    pub sequence: f64,
    pub cause: f64,
    pub factor: f64,
    pub finding: f64,
}
impl Default for Weights {
    fn default() -> Self {
        Self { sequence: 1.0, cause: 1.0, factor: 0.5, finding: 0.25 }
    }
}
#[derive(Deserialize)]
struct RawDict {
    #[serde(default)]
    weights: Option<Weights>,
    node: Vec<RawNode>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub tier: Tier,
    pub explains: Vec<String>,
    /// severity attribute (post-impact fire, evacuation): recorded on the graph, kept off the chain
    pub attribute: bool,
    /// node inserted by the builder when this one is present and the implied node's group is absent
    pub implies: Option<usize>,
}

pub struct Dict {
    pub nodes: Vec<Node>,
    pub weights: Weights,
    index: HashMap<String, usize>,
    /// sorted by priority (highest first), then file order
    matchers: Vec<(Source, Regex, usize)>,
}

/// Segment-aware prefix test: "mech.fuel" is a prefix of "mech.fuel.starvation" but not of "mech.fuelx".
/// The empty prefix matches everything.
pub fn has_prefix(id: &str, prefix: &str) -> bool {
    prefix.is_empty() || id == prefix || (id.starts_with(prefix) && id.as_bytes().get(prefix.len()) == Some(&b'.'))
}

/// Roll a node id up to its first `gran` segments (gran = 0 means full id).
pub fn bucket(id: &str, gran: usize) -> String {
    if gran == 0 {
        return id.to_string();
    }
    id.split('.').take(gran).collect::<Vec<_>>().join(".")
}

impl Dict {
    pub fn from_toml(text: &str) -> Result<Self> {
        let raw: RawDict = toml::from_str(text).context("parse causal_nodes.toml")?;
        let mut nodes = Vec::new();
        let mut index = HashMap::new();
        let mut matchers = Vec::new();
        let mut implies = Vec::new();
        for (i, n) in raw.node.into_iter().enumerate() {
            if index.insert(n.id.clone(), i).is_some() {
                bail!("duplicate node id {}", n.id);
            }
            for m in n.matchers {
                let re = Regex::new(&m.re).with_context(|| format!("node {} regex {}", n.id, m.re))?;
                matchers.push((m.prio, m.src, re, i));
            }
            implies.push(n.implies);
            nodes.push(Node { id: n.id, label: n.label, tier: n.tier, explains: n.explains, attribute: n.attribute, implies: None });
        }
        // stable: equal priorities keep file order
        matchers.sort_by_key(|m| std::cmp::Reverse(m.0));
        let matchers = matchers.into_iter().map(|(_, s, r, i)| (s, r, i)).collect();
        for (i, imp) in implies.into_iter().enumerate() {
            if let Some(id) = imp {
                let Some(&j) = index.get(&id) else { bail!("node {}: implies unknown node {id}", nodes[i].id) };
                nodes[i].implies = Some(j);
            }
        }
        for o in ["outcome.fatal", "outcome.serious", "outcome.minor", "outcome.none"] {
            if !index.contains_key(o) {
                bail!("dictionary must define {o}");
            }
        }
        let d = Self { nodes, weights: raw.weights.unwrap_or_default(), index, matchers };
        for n in &d.nodes {
            for p in &n.explains {
                if !d.nodes.iter().any(|m| has_prefix(&m.id, p)) {
                    bail!("node {}: explains prefix '{p}' matches no node", n.id);
                }
            }
        }
        Ok(d)
    }

    pub fn idx(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    /// Highest-priority matcher (then file order) for this source whose regex matches the text.
    pub fn map(&self, src: Source, text: &str) -> Option<usize> {
        self.matchers.iter().find(|(s, re, _)| *s == src && re.is_match(text)).map(|(_, _, i)| *i)
    }

    /// Can node `why` explain node `what` according to `why`'s explains prefixes?
    pub fn explains(&self, why: usize, what: usize) -> bool {
        let target = &self.nodes[what].id;
        self.nodes[why].explains.iter().any(|p| has_prefix(target, p))
    }
}
