//! WWCA demo on a SYNTHETIC dataset (no NTSB data). Shows every analysis the method defines.
//!   cargo run --release --bin wwca-demo -- [config/causal_nodes.toml] [out_dir]   (run from the workspace root)
//! Numbers printed here are made up by the generator below. Do not quote them.
use causal_chain::metrics::{self, Ctx, Depth, Query};
use causal_chain::{build, BuildOpts, Dict, FindRec, Injury, Involvement, OccRec, Role, Source, Tier};
use std::collections::BTreeMap;
use std::path::PathBuf;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[(self.next() % v.len() as u64) as usize]
    }
}

/// (text, role C/F/N, probability present)
type F08 = (&'static str, char, f64);
/// (text, role, linked occurrence number, probability)
type FPre = (&'static str, char, u32, f64);

struct Scenario {
    weight: f64,
    fatal_p: f64,
    classes: &'static [&'static str],
    occ08: &'static [&'static str],
    find08: &'static [F08],
    occ_pre: &'static [&'static str],
    find_pre: &'static [FPre],
}

const SCENARIOS: &[Scenario] = &[
    // fuel starvation -> total power loss -> forced landing -> collision
    Scenario { weight: 6.0, fatal_p: 0.10, classes: &["sep_fixed", "sep_retract", "mep"],
        occ08: &["Enroute-Loss of engine power (total)", "Emergency descent-Off-field or emergency landing", "Landing-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Fluids/misc hardware-Fluids-Fuel-Fuel starvation", 'C', 1.0),
                  ("Personnel issues-Task performance-Use of equip/info-Fuel tank selector position-Pilot", 'C', 0.8),
                  ("Personnel issues-Task performance-Use of equip/info-Checklist-Pilot", 'F', 0.2),
                  ("Environmental issues-Physical environment-Terrain-Trees-Effect on operation", 'F', 0.3)],
        occ_pre: &["LOSS OF ENGINE POWER(TOTAL) - NONMECHANICAL", "FORCED LANDING", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("FLUID,FUEL - STARVATION", 'C', 1, 1.0), ("FUEL TANK SELECTOR POSITION - IMPROPER - PILOT IN COMMAND", 'C', 1, 0.8),
                    ("TERRAIN CONDITION - TREES", 'F', 3, 0.3)] },
    // fuel exhaustion
    Scenario { weight: 7.0, fatal_p: 0.12, classes: &["sep_fixed", "sep_retract", "sep_tailwheel", "mep"],
        occ08: &["Enroute-Loss of engine power (total)", "Emergency descent-Off-field or emergency landing", "Landing-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Fluids/misc hardware-Fluids-Fuel-Fuel exhaustion", 'C', 1.0),
                  ("Personnel issues-Task performance-Planning/preparation-Fuel planning-Pilot", 'C', 0.9),
                  ("Personnel issues-Action/decision-Info processing/decision-Decision making/judgment-Pilot", 'F', 0.3)],
        occ_pre: &["LOSS OF ENGINE POWER(TOTAL) - NONMECHANICAL", "FORCED LANDING", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("FLUID,FUEL - EXHAUSTION", 'C', 1, 1.0), ("FUEL SUPPLY - INADEQUATE - PILOT IN COMMAND", 'C', 1, 0.9),
                    ("IN-FLIGHT PLANNING/DECISION - IMPROPER - PILOT IN COMMAND", 'F', 1, 0.3)] },
    // carburetor ice
    Scenario { weight: 3.0, fatal_p: 0.06, classes: &["sep_fixed", "sep_tailwheel"],
        occ08: &["Enroute-Loss of engine power (partial)", "Emergency descent-Off-field or emergency landing", "Landing-Nose over/nose down"],
        find08: &[("Aircraft-Aircraft power plant-Engine (reciprocating)-Induction system-Carburetor icing", 'C', 1.0),
                  ("Personnel issues-Task performance-Use of equip/info-Carburetor heat-Pilot", 'C', 0.7),
                  ("Environmental issues-Conditions/weather/phenomena-Conducive to carburetor icing-Effect on equipment", 'F', 0.8)],
        occ_pre: &["LOSS OF ENGINE POWER(PARTIAL) - NONMECHANICAL", "FORCED LANDING", "NOSE OVER"],
        find_pre: &[("CARBURETOR ICE", 'C', 1, 1.0), ("CARBURETOR HEAT - NOT USED - PILOT IN COMMAND", 'C', 1, 0.7),
                    ("WEATHER CONDITION - CARBURETOR ICING CONDITIONS", 'F', 1, 0.8)] },
    // engine mechanical
    Scenario { weight: 8.0, fatal_p: 0.08, classes: &["sep_fixed", "sep_retract", "mep", "experimental"],
        occ08: &["Enroute-Loss of engine power (total)", "Emergency descent-Off-field or emergency landing", "Landing-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Aircraft power plant-Engine (reciprocating)-Recip engine power section-Connecting rod-Fatigue/wear/corrosion", 'C', 1.0),
                  ("Personnel issues-Task performance-Inspection-Maintenance personnel", 'F', 0.35)],
        occ_pre: &["LOSS OF ENGINE POWER(TOTAL) - MECH FAILURE/MALF", "FORCED LANDING", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("ENGINE ASSEMBLY,CONNECTING ROD - FATIGUE", 'C', 1, 1.0),
                    ("MAINTENANCE,INSPECTION - INADEQUATE - COMPANY MAINTENANCE PERSONNEL", 'F', 1, 0.35)] },
    // undetermined power loss
    Scenario { weight: 5.0, fatal_p: 0.09, classes: &["sep_fixed", "sep_retract", "experimental"],
        occ08: &["Enroute-Loss of engine power (total)", "Emergency descent-Off-field or emergency landing", "Landing-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Aircraft power plant-Engine (reciprocating)-(general)-Unknown/Not determined", 'C', 1.0)],
        occ_pre: &["LOSS OF ENGINE POWER(TOTAL) - NONMECHANICAL", "FORCED LANDING", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("REASON FOR OCCURRENCE UNDETERMINED", 'C', 1, 1.0)] },
    // maneuvering stall
    Scenario { weight: 7.0, fatal_p: 0.60, classes: &["sep_fixed", "sep_tailwheel", "sep_aerobatic", "experimental"],
        occ08: &["Maneuvering-Aerodynamic stall/spin", "Uncontrolled descent-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Aircraft oper/perf/capability-Performance/control parameters-Angle of attack-Capability exceeded", 'C', 1.0),
                  ("Personnel issues-Aircraft control-Airspeed-Not attained/maintained-Pilot", 'C', 0.8),
                  ("Personnel issues-Action/decision-Action-Low altitude flight/maneuver-Pilot", 'C', 0.4),
                  ("Personnel issues-Physical-Alcohol/drug-Impairment(drug/alcohol)-Pilot", 'F', 0.08)],
        occ_pre: &["STALL/SPIN", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("STALL - INADVERTENT - PILOT IN COMMAND", 'C', 1, 1.0), ("AIRSPEED - NOT MAINTAINED - PILOT IN COMMAND", 'C', 1, 0.8),
                    ("LOW ALTITUDE FLIGHT/MANEUVER - PILOT IN COMMAND", 'C', 1, 0.4), ("ALCOHOL - IMPAIRED - PILOT IN COMMAND", 'F', 1, 0.08)] },
    // VFR into IMC -> LOC-I
    Scenario { weight: 4.0, fatal_p: 0.88, classes: &["sep_fixed", "sep_retract", "mep"],
        occ08: &["Enroute-VFR encounter with IMC", "Enroute-Loss of control in flight", "Uncontrolled descent-Collision with terr/obj (non-CFIT)"],
        find08: &[("Personnel issues-Psychological-Perception/orientation/illusion-Spatial disorientation-Pilot", 'C', 0.9),
                  ("Personnel issues-Action/decision-Info processing/decision-Decision making/judgment-Pilot", 'C', 0.8),
                  ("Environmental issues-Conditions/weather/phenomena-Ceiling/visibility/precip-Low ceiling", 'F', 0.9),
                  ("Personnel issues-Experience/knowledge-Total instrument experience-Pilot", 'F', 0.4),
                  ("Personnel issues-Task performance-Planning/preparation-Weather planning-Pilot", 'F', 0.4)],
        occ_pre: &["VFR FLIGHT INTO IMC", "LOSS OF CONTROL - IN FLIGHT", "IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("VFR FLIGHT INTO IMC - CONTINUED - PILOT IN COMMAND", 'C', 1, 0.8), ("WEATHER CONDITION - LOW CEILING", 'F', 1, 0.9),
                    ("SPATIAL DISORIENTATION - PILOT IN COMMAND", 'C', 2, 0.9), ("LACK OF TOTAL INSTRUMENT TIME - PILOT IN COMMAND", 'F', 2, 0.3)] },
    // CFIT on instrument approach
    Scenario { weight: 2.0, fatal_p: 0.80, classes: &["mep", "met", "sep_retract", "bizjet_light"],
        occ08: &["Approach-Controlled flight into terr/obj"],
        find08: &[("Personnel issues-Task performance-Use of equip/info-Instrument approach procedure-Pilot", 'C', 0.9),
                  ("Environmental issues-Conditions/weather/phenomena-Ceiling/visibility/precip-Fog", 'F', 0.8),
                  ("Environmental issues-Conditions/weather/phenomena-Light condition-Dark night", 'F', 0.5)],
        occ_pre: &["IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("IFR PROCEDURE - IMPROPER - PILOT IN COMMAND", 'C', 1, 0.9), ("WEATHER CONDITION - FOG", 'F', 1, 0.8),
                    ("LIGHT CONDITION - DARK NIGHT", 'F', 1, 0.5)] },
    // crosswind loss of control on ground
    Scenario { weight: 14.0, fatal_p: 0.003, classes: &["sep_tailwheel", "sep_fixed", "lsa"],
        occ08: &["Landing-Loss of control on ground", "Landing-Runway excursion"],
        find08: &[("Personnel issues-Aircraft control-Directional control-Pilot", 'C', 0.9),
                  ("Personnel issues-Task performance-Use of equip/info-Crosswind compensation-Pilot", 'C', 0.5),
                  ("Environmental issues-Conditions/weather/phenomena-Wind-Gusts-Effect on operation", 'F', 0.5),
                  ("Personnel issues-Experience/knowledge-Experience in type-Pilot", 'F', 0.15)],
        occ_pre: &["LOSS OF CONTROL - ON GROUND/WATER"],
        find_pre: &[("DIRECTIONAL CONTROL - NOT MAINTAINED - PILOT IN COMMAND", 'C', 1, 0.9),
                    ("COMPENSATION FOR WIND CONDITIONS - INADEQUATE - PILOT IN COMMAND", 'C', 1, 0.5),
                    ("WEATHER CONDITION - GUSTS", 'F', 1, 0.5)] },
    // hard landing
    Scenario { weight: 8.0, fatal_p: 0.0, classes: &["sep_fixed", "lsa", "sep_tailwheel"],
        occ08: &["Landing-Hard landing"],
        find08: &[("Personnel issues-Task performance-Use of equip/info-Landing flare-Pilot", 'C', 0.9),
                  ("Personnel issues-Task performance-Use of equip/info-Checklist-Pilot", 'F', 0.05)],
        occ_pre: &["HARD LANDING"],
        find_pre: &[("FLARE - IMPROPER - PILOT IN COMMAND", 'C', 1, 0.9)] },
    // gear up
    Scenario { weight: 4.0, fatal_p: 0.0, classes: &["sep_retract", "mep"],
        occ08: &["Landing-Landing gear not configured"],
        find08: &[("Personnel issues-Task performance-Use of equip/info-Landing gear extension-Pilot", 'C', 0.9),
                  ("Personnel issues-Task performance-Use of equip/info-Checklist-Pilot", 'F', 0.5)],
        occ_pre: &["WHEELS UP LANDING"],
        find_pre: &[("LANDING GEAR EXTENSION - NOT PERFORMED - PILOT IN COMMAND", 'C', 1, 0.9), ("CHECKLIST - NOT FOLLOWED - PILOT IN COMMAND", 'F', 1, 0.5)] },
    // midair
    Scenario { weight: 1.0, fatal_p: 0.70, classes: &["sep_fixed", "sep_retract"],
        occ08: &["Enroute-Midair collision"],
        find08: &[("Personnel issues-Task performance-Monitoring environment-Visual lookout-Pilot", 'C', 0.9),
                  ("Environmental issues-Physical environment-Object/animal/substance-Other aircraft-Effect on operation", 'F', 0.7)],
        occ_pre: &["MIDAIR COLLISION"],
        find_pre: &[("VISUAL LOOKOUT - INADEQUATE - PILOT IN COMMAND", 'C', 1, 0.9)] },
    // density-altitude takeoff
    Scenario { weight: 2.0, fatal_p: 0.30, classes: &["sep_fixed", "sep_tailwheel"],
        occ08: &["Initial climb-Collision with terr/obj (non-CFIT)"],
        find08: &[("Aircraft-Aircraft oper/perf/capability-Performance/control parameters-Climb capability-Not attained/maintained", 'C', 0.9),
                  ("Environmental issues-Conditions/weather/phenomena-High density altitude-Effect on operation", 'C', 0.8),
                  ("Personnel issues-Task performance-Planning/preparation-Performance calculations-Pilot", 'C', 0.6)],
        occ_pre: &["IN FLIGHT COLLISION WITH TERRAIN/WATER"],
        find_pre: &[("AIRCRAFT PERFORMANCE,CLIMB CAPABILITY - EXCEEDED", 'C', 1, 0.9), ("DENSITY ALTITUDE - HIGH", 'F', 1, 0.8)] },
];

fn role(c: char) -> Role {
    match c {
        'C' => Role::Cause,
        'F' => Role::Factor,
        _ => Role::Finding,
    }
}

fn generate(n: usize, seed: u64) -> Vec<Involvement> {
    let mut rng = Rng(seed);
    let total: f64 = SCENARIOS.iter().map(|s| s.weight).sum();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut x = rng.f() * total;
        let s = SCENARIOS.iter().find(|s| {
            x -= s.weight;
            x <= 0.0
        }).unwrap_or(&SCENARIOS[0]);
        let modern = rng.f() < 0.5;
        let year = if modern { 2008 + (rng.next() % 18) } else { 1982 + (rng.next() % 26) };
        let mut attrs = BTreeMap::new();
        attrs.insert("class".into(), rng.pick(s.classes).to_string());
        attrs.insert("year".into(), year.to_string());
        attrs.insert("era".into(), if modern { "2008+" } else { "pre2008" }.into());
        let injury = if rng.f() < s.fatal_p { Injury::Fatal } else if rng.f() < 0.15 { Injury::Serious } else { Injury::None };
        let (occurrences, findings) = if modern {
            let occ = s.occ08.iter().enumerate().map(|(k, t)| OccRec {
                seq: k as u32 + 1, src: Source::Occ08,
                text: t.split_once('-').map(|x| x.1).unwrap_or(t).to_string(), defining: k == 0 }).collect();
            let f = s.find08.iter().filter(|(_, _, p)| rng.f() < *p)
                .map(|(t, r, _)| FindRec { src: Source::Find08, text: t.to_string(), role: role(*r), occ_link: None }).collect();
            (occ, f)
        } else {
            let occ = s.occ_pre.iter().enumerate().map(|(k, t)| OccRec { seq: k as u32 + 1, src: Source::OccPre08, text: t.to_string(), defining: false }).collect();
            let f = s.find_pre.iter().filter(|(_, _, _, p)| rng.f() < *p)
                .map(|(t, r, l, _)| FindRec { src: Source::FindPre08, text: t.to_string(), role: role(*r), occ_link: Some(*l) }).collect();
            (occ, f)
        };
        out.push(Involvement { id: format!("SYN{i:06}/1"), attrs, injury, occurrences, findings });
    }
    out
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dict_path = args.get(1).cloned().unwrap_or_else(|| "config/causal_nodes.toml".into());
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "wwca_demo_out".into()));
    std::fs::create_dir_all(&out)?;
    let dict = Dict::from_toml(&std::fs::read_to_string(&dict_path)?)?;
    let invs = generate(20_000, 42);
    let graphs: Vec<_> = invs.iter().map(|i| build(&dict, i, &BuildOpts::default())).collect();
    let all: Vec<&_> = graphs.iter().collect();

    println!("=== SYNTHETIC DATA — illustrates the method only ===\n");
    println!("Example chain (involvement 0):\n{}", causal_chain::build::render(&dict, &graphs[0]));
    let q = metrics::qa(&dict, &all, 10);
    println!("QA: {} involvements, {} unmapped rows, {} inversions, edges by provenance {:?}", q.involvements, q.unmapped_rows, q.inversions, q.edges_by_prov);
    println!("    no end node {}, roles uncoded {}, spines reordered {}, implied nodes {}\n", q.no_end, q.roles_uncoded, q.reordered, q.implied_nodes);
    for (t, n) in &q.unmapped_top {
        println!("  unmapped {n:>6}  {t}");
    }

    // 1. Global: what % of WHATs and WHYs, by tier
    for (name, tiers, gran) in [
        ("tier_end", vec![Tier::End], 0),
        ("tier_critical", vec![Tier::Critical], 0),
        ("tier_mechanism", vec![Tier::Mechanism, Tier::Undetermined], 2),
        ("tier_act", vec![Tier::Act], 0),
        ("tier_latent", vec![Tier::Latent], 0),
        ("tier_context", vec![Tier::Context], 0),
    ] {
        let b = metrics::tier_overview(&dict, &all, &tiers, gran, 200, 1);
        println!("{}", metrics::print_breakdown(&b, 12));
        metrics::write_breakdown_csv(&out.join(format!("{name}.csv")), &b)?;
    }

    // 2. Drill: WHY of total power loss (direct), then the root whys
    let mut qd = Query::new("crit.power_loss");
    println!("{}", metrics::print_breakdown(&metrics::why_breakdown(&dict, &all, &qd), 12));
    metrics::write_breakdown_csv(&out.join("why_power_loss_direct.csv"), &metrics::why_breakdown(&dict, &all, &qd))?;
    qd.depth = Depth::Root;
    qd.ctx = Ctx::Exclude;
    println!("{}", metrics::print_breakdown(&metrics::why_breakdown(&dict, &all, &qd), 12));

    // 3. Drill one level down: WHY of fuel starvation/exhaustion (rolled up to mech.fuel)
    let qf = Query { focal: "mech.fuel".into(), ..Query::new("") };
    println!("{}", metrics::print_breakdown(&metrics::why_breakdown(&dict, &all, &qf), 12));

    // 4. Reverse: WHAT does pilot decision-making lead to (any depth)?
    let qr = Query { depth: Depth::Any, ..Query::new("act.pilot.decision") };
    println!("{}", metrics::print_breakdown(&metrics::what_breakdown(&dict, &all, &qr), 12));

    // 5. Fatal-only cohort: WHY of the outcome (direct = how fatal flights ended)
    let fatal: Vec<&_> = graphs.iter().filter(|g| g.fatal).collect();
    let qo = Query { gran: 2, ..Query::new("outcome") };
    println!("{}", metrics::print_breakdown(&metrics::why_breakdown(&dict, &fatal, &qo), 12));

    // 6. Lethality of critical events
    println!("LETHALITY (critical tier)");
    for r in metrics::lethality(&dict, &all, &[Tier::Critical], 0) {
        println!("  {:<28} n={:>6} fatal={:>5.1}% [{:>4.1},{:>4.1}] lift={:.2}", r.bucket, r.n, 100.0 * r.rate, 100.0 * r.ci.0, 100.0 * r.ci.1, r.lift);
    }

    // 7. Chain signatures
    println!("\nTOP CHAIN SIGNATURES (gran 2, context excluded)");
    let ch = metrics::chains(&dict, &all, 2, Ctx::Exclude, 12);
    let mut w = csv::Writer::from_path(out.join("chains.csv"))?;
    w.write_record(["signature", "n", "share", "fatal_rate"])?;
    for r in &ch {
        println!("  {:>5.1}%  fatal {:>5.1}%  {}", 100.0 * r.share, 100.0 * r.fatal_rate, r.signature);
        w.write_record([r.signature.clone(), r.n.to_string(), format!("{:.4}", r.share), format!("{:.4}", r.fatal_rate)])?;
    }
    w.flush()?;

    // 8. Flow (Sankey input)
    let fl = metrics::flow(&dict, &all, 2, Ctx::Include);
    let mut w = csv::Writer::from_path(out.join("flow_gran2.csv"))?;
    w.write_record(["what", "why", "n", "share_of_what", "share_of_why"])?;
    for r in &fl {
        w.write_record([r.what.clone(), r.why.clone(), r.n.to_string(), format!("{:.4}", r.share_of_what), format!("{:.4}", r.share_of_why)])?;
    }
    w.flush()?;

    // 9. Era sensitivity: same question, two coding systems
    let co = metrics::Cohort::parse("era=2008+");
    let cp = metrics::Cohort::parse("era=pre2008");
    let a: Vec<&_> = graphs.iter().filter(|g| co.matches(&g.attrs)).collect();
    let b: Vec<&_> = graphs.iter().filter(|g| cp.matches(&g.attrs)).collect();
    let qq = Query { bootstrap: 0, ..Query::new("crit.power_loss") };
    println!("\nERA COMPARE: WHY of crit.power_loss (2008+ vs pre-2008)");
    for r in metrics::compare(&metrics::why_breakdown(&dict, &a, &qq), &metrics::why_breakdown(&dict, &b, &qq)) {
        println!("  {:<28} {:>5.1}% vs {:>5.1}%  q={:.3}", r.bucket, 100.0 * r.inc_a, 100.0 * r.inc_b, r.q);
    }
    println!("\nCSV written to {}", out.display());
    Ok(())
}
