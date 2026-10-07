//! Contract test: every row of tests/fixtures/taxonomy_golden.csv must classify as expected.
use aircraft_taxonomy::{MatchResult, Taxonomy};

#[test]
fn golden_cases_pass() {
    let tax = Taxonomy::from_toml(include_str!("../../../config/aircraft_types.toml")).expect("taxonomy loads");
    let mut rdr = csv::Reader::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/taxonomy_golden.csv")).unwrap();
    let mut fails = Vec::new();
    let mut n = 0;
    for rec in rdr.records() {
        let r = rec.unwrap();
        n += 1;
        let got = tax.classify(&r[0], &r[1], &r[2], &r[3] == "Y");
        let (ef, ev) = (&r[4], &r[5]);
        let ok = match &got {
            MatchResult::Matched { family, variant, .. } => family == ef && variant.as_deref().unwrap_or("") == ev,
            MatchResult::NoMatch => ef.is_empty(),
            MatchResult::Ambiguous(_) => false,
        };
        if !ok {
            fails.push(format!("{:?} -> {:?} (expected {ef}/{ev})", r, got));
        }
    }
    assert!(n > 0);
    assert!(fails.is_empty(), "{} of {n} failed:\n{}", fails.len(), fails.join("\n"));
}
