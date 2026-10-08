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
    /// CausesAndFactors when the involvement has any finding coded C or F, otherwise All.
    /// CAROL-era (2021+) findings carry no Cause_Factor, so a fixed CausesAndFactors filter would drop them all.
    Auto,
    CausesOnly,
    CausesAndFactors,
    All,
}

impl RoleFilter {
    /// Resolve `Auto` for one involvement. Other filters come back unchanged.
    pub fn resolve(self, inv: &Involvement) -> RoleFilter {
        match self {
            RoleFilter::Auto if inv.roles_coded() => RoleFilter::CausesAndFactors,
            RoleFilter::Auto => RoleFilter::All,
            f => f,
        }
    }

    /// Call on a resolved filter; an unresolved `Auto` behaves as CausesAndFactors.
    pub fn keep(self, r: Role) -> bool {
        match self {
            RoleFilter::All => true,
            RoleFilter::Auto | RoleFilter::CausesAndFactors => r != Role::Finding,
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
    /// 2008+: event part of the description (`text::normalize_occ08`); pre-2008: decoded occurrence text
    pub text: String,
    /// avall events_sequence.Defining_ev
    pub defining: bool,
}

#[derive(Clone, Debug)]
pub struct FindRec {
    pub src: Source,
    /// 2008+: role suffix stripped (`text::split_find08`)
    pub text: String,
    /// Cause_Factor; for 2008–2020 rows with a blank Cause_Factor, the text's " - C"/" - F" suffix
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

impl Involvement {
    /// Does any finding carry an NTSB cause/factor role?
    pub fn roles_coded(&self) -> bool {
        self.findings.iter().any(|f| matches!(f.role, Role::Cause | Role::Factor))
    }
}
