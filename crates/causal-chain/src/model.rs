//! Input records: one `Involvement` per (ev_id, Aircraft_Key).
use crate::dict::Source;
use std::collections::BTreeMap;

/// NTSB Cause_Factor role of a finding. Occurrence rows carry `Sequence`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Finding,
    Factor,
    Cause,
    Sequence,
}

impl Role {
    /// Parse NTSB Cause_Factor ("C", "F", blank).
    pub fn from_ntsb(cf: &str) -> Self {
        match cf.trim() {
            "C" | "c" => Role::Cause,
            "F" | "f" => Role::Factor,
            _ => Role::Finding,
        }
    }
}

/// Which finding roles take part in the analysis. Occurrences always take part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleFilter {
    CausesOnly,
    CausesAndFactors,
    All,
}

impl RoleFilter {
    pub fn keep(self, r: Role) -> bool {
        match self {
            RoleFilter::All => true,
            RoleFilter::CausesAndFactors => r != Role::Finding,
            RoleFilter::CausesOnly => matches!(r, Role::Cause | Role::Sequence),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Injury {
    Fatal,
    Serious,
    Minor,
    None,
}

impl Injury {
    pub fn node_id(self) -> &'static str {
        match self {
            Injury::Fatal => "outcome.fatal",
            Injury::Serious => "outcome.serious",
            Injury::Minor => "outcome.minor",
            Injury::None => "outcome.none",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OccRec {
    /// Occurrence_No (sequence order within the aircraft)
    pub seq: u32,
    pub src: Source,
    /// 2008+: event part of "Phase-Event"; pre-2008: decoded occurrence text
    pub text: String,
    /// avall events_sequence.Defining_ev
    pub defining: bool,
}

#[derive(Clone, Debug)]
pub struct FindRec {
    pub src: Source,
    pub text: String,
    pub role: Role,
    /// pre-2008 seq_of_events.Occurrence_No (explicit link); None for 2008+
    pub occ_link: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Involvement {
    /// "<ev_id>/<Aircraft_Key>"
    pub id: String,
    /// cohort attributes: class, family, year, far_part, era, phase, ...
    pub attrs: BTreeMap<String, String>,
    pub injury: Injury,
    pub occurrences: Vec<OccRec>,
    pub findings: Vec<FindRec>,
}
