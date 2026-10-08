//! Normalisation of raw avall texts before dictionary matching (spec §3).
use crate::model::Role;

/// CAROL/eADMS flight phases as they prefix `Events_Sequence.Occurrence_Description`.
/// The description is "<phase> <event>" joined by a SPACE; phases themselves contain hyphens.
pub const PHASES_08: &[&str] = &[
    "Prior to flight", "Standing", "Standing-engine(s) not oper", "Standing-engine(s) start-up",
    "Standing-engine(s) operating", "Standing-engine(s) shutdown", "Pushback/towing", "Pushback/tow-engine not oper",
    "Pushback/tow-engine start-up", "Pushback/tow-engine oper", "Taxi", "Taxi-to runway", "Taxi-into takeoff position",
    "Taxi-from runway", "Takeoff", "Takeoff-rejected takeoff", "Initial climb", "Enroute", "Enroute-climb to cruise",
    "Enroute-cruise", "Enroute-change of cruise level", "Enroute-descent", "Enroute-holding (IFR)", "Maneuvering",
    "Maneuvering-aerobatics", "Maneuvering-low-alt flying", "Maneuvering-hover", "Approach",
    "Approach-IFR initial approach", "Approach-IFR final approach", "Approach-circling (IFR)",
    "Approach-IFR missed approach", "Approach-VFR pattern crosswind", "Approach-VFR pattern downwind",
    "Approach-VFR pattern base", "Approach-VFR pattern final", "Approach-VFR go-around", "Landing",
    "Landing-flare/touchdown", "Landing-landing roll", "Landing-aborted after touchdown", "Emergency descent",
    "Autorotation", "Uncontrolled descent", "Post-impact", "After landing", "Other", "Unknown",
];

/// CAROL/eADMS occurrence event names (`eventsoe_no` 000-990), as they end the description.
pub const EVENTS_08: &[&str] = &[
    "Unknown or undetermined", "Aircraft loading event", "Aircraft servicing event", "Preflight or dispatch event",
    "Aircraft maintenance event", "Aircraft inspection event", "Attempted remediation/recovery",
    "Airport occurrence", "Ground handling event", "AC/prop/rotor contact w person", "Prop/jet/rotor blast/suction",
    "Abnormal runway contact", "Tailstrike", "Hard landing", "Dragged wing/rotor/float/other",
    "Landing gear collapse", "Landing gear not configured", "Nose over/nose down", "Roll over", "Air traffic event",
    "Cabin safety event", "Controlled flight into terr/obj (CFIT)", "Emergency descent initiated",
    "Engine shutdown", "Fire/smoke (non-impact)", "Explosion (non-impact)", "Fire/smoke (post-impact)",
    "Explosion (post-impact)", "Fuel related", "Fuel starvation", "Fuel exhaustion", "Fuel contamination",
    "Wrong fuel", "Ground collision", "Structural icing", "Low altitude operation/event",
    "Loss of control on ground", "Dynamic rollover", "Ground resonance", "Loss of control in flight",
    "Aerodynamic stall/spin", "Loss of tail rotor effectiveness", "Retreating blade stall",
    "Settling with power/vortex ring state", "Mast bumping", "Midair collision",
    "Near midair/TCAS alert/loss of separation", "Abrupt maneuver", "Inflight upset", "Navigation error",
    "Course deviation", "Altitude deviation", "Airspace incursion", "Wrong surface or wrong airport",
    "Runway excursion", "Wildlife encounter (non-bird)", "Runway incursion veh/AC/person",
    "Sys/Comp malf/fail (non-power)", "Pressure/environ sys malf/fail", "Electrical system malf/failure",
    "Flight control sys malf/fail", "Flight instrument malf/fail", "Nav system malfunction/failure",
    "Comm system malf/failure", "Aircraft structural failure", "Part(s) separation from AC",
    "Powerplant sys/comp malf/fail", "Loss of engine power (total)", "Loss of engine power (partial)",
    "Uncontained engine failure", "Security/criminal event", "Turbulence encounter", "Aircraft wake turb encounter",
    "Clear air turbulence encounter", "Landing area undershoot", "Landing area overshoot",
    "Windshear or thunderstorm", "Other weather encounter", "VFR encounter with IMC", "Loss of visual reference",
    "Terrain avoidance alert", "Collision avoidance alert", "Stall warn/stick-shaker/pusher",
    "Off-field or emergency landing", "Ditching", "Hazardous material leak/spill", "Evacuation",
    "Collision with terr/obj (non-CFIT)", "External load event (Rotorcraft)", "Collision during takeoff/land",
    "Loss of lift", "Glider tow event", "Simulated/training event", "Medical event", "Miscellaneous/other",
    "Birdstrike", "Missing aircraft",
];

/// Split an avall occurrence description into (phase, event). A known event suffix wins, so phase names that are
/// also event words ("Landing gear collapse", "Unknown or undetermined") are not cut; otherwise the longest
/// phase prefix is stripped. Idempotent: an event text alone comes back as ("", event).
pub fn split_occ08(desc: &str) -> (&str, &str) {
    let desc = desc.trim();
    let is_phase = |p: &str| p.is_empty() || PHASES_08.contains(&p);
    if let Some(ev) = EVENTS_08
        .iter()
        .filter(|e| desc.ends_with(**e) && is_phase(desc[..desc.len() - e.len()].trim_end()))
        .max_by_key(|e| e.len())
    {
        let i = desc.len() - ev.len();
        return (desc[..i].trim_end(), &desc[i..]);
    }
    PHASES_08
        .iter()
        .filter(|p| desc.len() > p.len() && desc.starts_with(**p) && desc.as_bytes()[p.len()] == b' ')
        .max_by_key(|p| p.len())
        .map(|p| (&desc[..p.len()], desc[p.len() + 1..].trim()))
        .unwrap_or(("", desc))
}

/// Event part of an occurrence description: the text `occ08` matchers see.
pub fn normalize_occ08(desc: &str) -> &str {
    split_occ08(desc).1
}

/// Split a 2008+ finding description into (text, role suffix). About half of the 2008–2020 rows end in
/// " - C" or " - F", repeating Cause_Factor inside the text.
pub fn split_find08(desc: &str) -> (&str, Option<Role>) {
    let d = desc.trim();
    match d.len().checked_sub(4).map(|i| (&d[..i], &d[i..])) {
        Some((head, " - C")) => (head.trim_end(), Some(Role::Cause)),
        Some((head, " - F")) => (head.trim_end(), Some(Role::Factor)),
        _ => (d, None),
    }
}

/// Finding text without the role suffix: the text `find08` matchers see.
pub fn normalize_find08(desc: &str) -> &str {
    split_find08(desc).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_phase_and_event() {
        assert_eq!(split_occ08("Initial climb Loss of control in flight"), ("Initial climb", "Loss of control in flight"));
        assert_eq!(split_occ08("Landing-landing roll Runway excursion"), ("Landing-landing roll", "Runway excursion"));
        assert_eq!(split_occ08("Takeoff-rejected takeoff Runway excursion"), ("Takeoff-rejected takeoff", "Runway excursion"));
        assert_eq!(split_occ08("Post-impact Fire/smoke (post-impact)"), ("Post-impact", "Fire/smoke (post-impact)"));
        assert_eq!(split_occ08("Other Miscellaneous/other"), ("Other", "Miscellaneous/other"));
        assert_eq!(split_occ08("Loss of engine power (total)"), ("", "Loss of engine power (total)"));
        assert_eq!(split_occ08("Landing Landing gear collapse"), ("Landing", "Landing gear collapse"));
        assert_eq!(split_occ08("Landing gear collapse"), ("", "Landing gear collapse"));
        assert_eq!(split_occ08("Unknown Unknown or undetermined"), ("Unknown", "Unknown or undetermined"));
        assert_eq!(split_occ08("Unknown or undetermined"), ("", "Unknown or undetermined"));
        assert_eq!(split_occ08("Other Other weather encounter"), ("Other", "Other weather encounter"));
    }

    #[test]
    fn strips_role_suffix() {
        assert_eq!(split_find08("Personnel issues-Aircraft control-Pilot - C"), ("Personnel issues-Aircraft control-Pilot", Some(Role::Cause)));
        assert_eq!(split_find08("Environmental issues-Wind-Effect on operation - F"), ("Environmental issues-Wind-Effect on operation", Some(Role::Factor)));
        assert_eq!(split_find08("Personnel issues-Aircraft control-Pilot"), ("Personnel issues-Aircraft control-Pilot", None));
    }
}
