use causal_chain::metrics::{self, Ctx, Depth, Query};
use causal_chain::*;
use std::collections::BTreeMap;

fn dict() -> Dict {
    Dict::from_toml(include_str!("../../../config/causal_nodes.toml")).expect("dictionary loads")
}

fn occ08(seq: u32, t: &str) -> OccRec {
    OccRec { seq, src: Source::Occ08, text: t.into(), defining: seq == 1 }
}
fn f08(t: &str, r: Role) -> FindRec {
    FindRec { src: Source::Find08, text: t.into(), role: r, occ_link: None }
}
fn inv(id: &str, injury: Injury, occurrences: Vec<OccRec>, findings: Vec<FindRec>) -> Involvement {
    Involvement { id: id.into(), attrs: BTreeMap::new(), injury, occurrences, findings }
}

/// Dawson's example: fatal forced-landing collision <- power loss <- fuel starvation <- pilot fuel error.
fn user_example() -> Involvement {
    inv(
        "EX1/1",
        Injury::Fatal,
        vec![occ08(1, "Loss of engine power (total)"), occ08(2, "Off-field or emergency landing"), occ08(3, "Collision with terr/obj (non-CFIT)")],
        vec![
            f08("Aircraft-Fluids/misc hardware-Fluids-Fuel-Fuel starvation", Role::Cause),
            f08("Personnel issues-Task performance-Use of equip/info-Fuel tank selector position-Pilot", Role::Cause),
            f08("Environmental issues-Physical environment-Terrain-Trees-Effect on operation", Role::Factor),
        ],
    )
}

fn ids(d: &Dict, g: &Graph, v: &[usize]) -> Vec<String> {
    let mut x: Vec<String> = v.iter().map(|&i| d.nodes[g.nodes[i].node].id.clone()).collect();
    x.sort();
    x
}

#[test]
fn user_example_builds_expected_chain() {
    let d = dict();
    let g = build(&d, &user_example(), &BuildOpts::default());
    let tree = causal_chain::build::render(&d, &g);
    let expected = "\
outcome.fatal
  end.collision  <Outcome>
    end.forced_landing  <Sequence>
      crit.power_loss.total  <Sequence>
        mech.fuel.starvation [C]  <Rule>
          act.pilot.fuel_mgmt [C]  <Rule>
      env.terrain [F]  <Rule>
";
    assert_eq!(tree, expected, "\n{tree}");
    assert!(g.unmapped.is_empty());
}

#[test]
fn what_why_is_relative() {
    let d = dict();
    let g = build(&d, &user_example(), &BuildOpts::default());
    let q = |focal: &str, depth: Depth, ctx: Ctx| Query { depth, ctx, ..Query::new(focal) };
    let whys = |focal: &str, depth: Depth, ctx: Ctx| {
        let f = metrics::focal_set(&d, &g, focal);
        ids(&d, &g, &metrics::collect_whys(&d, &g, &f, &q(focal, depth, ctx)))
    };
    // WHAT = emergency landing collision -> WHY = forced landing
    assert_eq!(whys("end.collision", Depth::Direct, Ctx::Include), vec!["end.forced_landing"]);
    // WHAT = forced landing -> WHY = power loss (+ trees as context)
    assert_eq!(whys("end.forced_landing", Depth::Direct, Ctx::Include), vec!["crit.power_loss.total", "env.terrain"]);
    // WHAT = power loss -> WHY = fuel starvation
    assert_eq!(whys("crit.power_loss", Depth::Direct, Ctx::Include), vec!["mech.fuel.starvation"]);
    // WHAT = fuel starvation -> WHY = pilot fuel management
    assert_eq!(whys("mech.fuel", Depth::Direct, Ctx::Include), vec!["act.pilot.fuel_mgmt"]);
    // root why of the power loss
    assert_eq!(whys("crit.power_loss", Depth::Root, Ctx::Exclude), vec!["act.pilot.fuel_mgmt"]);
    // focal bucket "end" passes through its own members: first why outside the bucket
    assert_eq!(whys("end", Depth::Direct, Ctx::Exclude), vec!["crit.power_loss.total"]);
    // reverse: what does the pilot fuel error lead to, any depth (outcome included at graph level)
    let f = metrics::focal_set(&d, &g, "act.pilot.fuel_mgmt");
    let whats = ids(&d, &g, &metrics::collect_whats(&d, &g, &f, &Query { depth: Depth::Any, ..Query::new("act.pilot.fuel_mgmt") }));
    assert_eq!(whats, vec!["crit.power_loss.total", "end.collision", "end.forced_landing", "mech.fuel.starvation", "outcome.fatal"]);
}

#[test]
fn pre2008_explicit_links() {
    let d = dict();
    let occ = |seq: u32, t: &str| OccRec { seq, src: Source::OccPre08, text: t.into(), defining: false };
    let f = |t: &str, r: Role, l: u32| FindRec { src: Source::FindPre08, text: t.into(), role: r, occ_link: Some(l) };
    let i = inv(
        "PRE/1",
        Injury::None,
        vec![occ(1, "LOSS OF ENGINE POWER(TOTAL) - NONMECHANICAL"), occ(2, "FORCED LANDING"), occ(3, "IN FLIGHT COLLISION WITH TERRAIN/WATER")],
        vec![
            f("FLUID,FUEL - STARVATION", Role::Cause, 1),
            f("FUEL TANK SELECTOR POSITION - IMPROPER - PILOT IN COMMAND", Role::Cause, 1),
            f("TERRAIN CONDITION - TREES", Role::Finding, 3),
        ],
    );
    let g = build(&d, &i, &BuildOpts { roles: RoleFilter::All });
    let tree = causal_chain::build::render(&d, &g);
    let expected = "\
outcome.none
  end.collision  <Outcome>
    end.forced_landing  <Sequence>
      crit.power_loss.total  <Sequence>
        mech.fuel.starvation [C]  <Explicit>
          act.pilot.fuel_mgmt [C]  <Explicit>
    env.terrain [-]  <Explicit>
";
    assert_eq!(tree, expected, "\n{tree}");
    // with the default role filter the plain finding is dropped
    let g2 = build(&d, &i, &BuildOpts::default());
    assert!(!causal_chain::build::render(&d, &g2).contains("env.terrain"));
}

#[test]
fn default_attachment_when_no_rule() {
    let d = dict();
    let i = inv(
        "HL/1",
        Injury::None,
        vec![occ08(1, "Hard landing")],
        vec![f08("Personnel issues-Task performance-Use of equip/info-Checklist-Pilot", Role::Factor)],
    );
    let g = build(&d, &i, &BuildOpts::default());
    let e = g.edges.iter().find(|e| d.nodes[g.nodes[e.why].node].id == "act.pilot.checklist").unwrap();
    assert_eq!(e.prov, Prov::Default);
    assert_eq!(d.nodes[g.nodes[e.what].node].id, "end.hard_landing");
}

#[test]
fn sequence_inversion_is_repaired_by_rule() {
    let d = dict();
    // coded out of order: collision listed before the power loss
    let i = inv("INV/1", Injury::Minor, vec![occ08(1, "Collision with terr/obj (non-CFIT)"), occ08(2, "Loss of engine power (total)")], vec![]);
    let g = build(&d, &i, &BuildOpts::default());
    assert_eq!(g.inversions, 1);
    let tree = causal_chain::build::render(&d, &g);
    assert_eq!(tree, "outcome.minor\n  end.collision  <Outcome>\n    crit.power_loss.total  <Rule>\n");
}

#[test]
fn every_node_has_exactly_one_what() {
    let d = dict();
    let g = build(&d, &user_example(), &BuildOpts { roles: RoleFilter::All });
    for n in 0..g.nodes.len() {
        let k = g.edges.iter().filter(|e| e.why == n).count();
        assert_eq!(k, if n == g.outcome { 0 } else { 1 }, "node {n}");
    }
}

fn mixed_sample() -> Vec<Involvement> {
    let mut v = Vec::new();
    for k in 0..60u32 {
        let mut f = vec![];
        if k % 2 == 0 {
            f.push(f08("Aircraft-Fluids/misc hardware-Fluids-Fuel-Fuel exhaustion", Role::Cause));
        }
        if k % 3 == 0 {
            f.push(f08("Personnel issues-Task performance-Planning/preparation-Fuel planning-Pilot", Role::Cause));
        }
        if k % 5 == 0 {
            f.push(f08("Personnel issues-Action/decision-Info processing/decision-Decision making/judgment-Pilot", Role::Factor));
        }
        if k % 7 == 0 {
            f.push(f08("Aircraft-Aircraft power plant-Engine (reciprocating)-Recip engine power section-Cylinder-Fatigue/wear/corrosion", Role::Cause));
        }
        v.push(inv(
            &format!("S{k}/1"),
            if k % 4 == 0 { Injury::Fatal } else { Injury::None },
            vec![occ08(1, "Loss of engine power (total)"), occ08(2, "Off-field or emergency landing")],
            f,
        ));
    }
    v
}

#[test]
fn attribution_sums_to_one_and_never_exceeds_incidence() {
    let d = dict();
    let gs: Vec<Graph> = mixed_sample().iter().map(|i| build(&d, i, &BuildOpts::default())).collect();
    let refs: Vec<&Graph> = gs.iter().collect();
    for depth in [Depth::Direct, Depth::Exactly(2), Depth::Root, Depth::Any] {
        for gran in [0, 1, 2] {
            let b = metrics::why_breakdown(&d, &refs, &Query { depth, gran, bootstrap: 50, ..Query::new("crit") });
            let s: f64 = b.rows.iter().map(|r| r.attribution).sum();
            assert!((s - 1.0).abs() < 1e-9, "{depth:?} gran {gran}: sum {s}");
            for r in &b.rows {
                assert!(r.attribution <= r.incidence + 1e-12, "{}", r.bucket);
                assert!(r.attr_ci.0 <= r.attribution + 1e-9 && r.attribution <= r.attr_ci.1 + 1e-9, "{} ci", r.bucket);
            }
        }
    }
    let t = metrics::tier_overview(&d, &refs, &[Tier::Act], 0, 0, 1);
    assert!((t.rows.iter().map(|r| r.attribution).sum::<f64>() - 1.0).abs() < 1e-9);
}

#[test]
fn cohort_filter() {
    let mut a = BTreeMap::new();
    a.insert("class".to_string(), "sep_fixed".to_string());
    a.insert("year".to_string(), "2012".to_string());
    assert!(metrics::Cohort::parse("class=sep_fixed|sep_retract;year>=2008").matches(&a));
    assert!(!metrics::Cohort::parse("year<2008").matches(&a));
    assert!(!metrics::Cohort::parse("far_part=121").matches(&a));
}
