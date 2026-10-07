//! Aircraft taxonomy loader + matcher. Implements the matching contract in aircraft_types.toml.
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize)]
struct RawFile {
    makes: BTreeMap<String, Vec<String>>,
    #[serde(rename = "class")]
    classes: Vec<RawClass>,
    #[serde(rename = "family")]
    families: Vec<RawFamily>,
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct RawClass {
    pub id: String,
    pub name: String,
}
#[derive(Deserialize)]
struct RawFamily {
    id: String,
    class: String,
    makes: Vec<String>,
    model_re: String,
    homebuilt_model_re: Option<String>,
    #[serde(default = "default_priority")]
    priority: i32,
    #[serde(default)]
    variant: Vec<RawVariant>,
}
#[derive(Deserialize)]
struct RawVariant {
    id: String,
    model_re: String,
    class: Option<String>,
}
fn default_priority() -> i32 {
    100
}

pub struct Variant {
    pub id: String,
    pub class: Option<String>,
    re: Regex,
}
pub struct Family {
    pub id: String,
    pub class: String,
    pub priority: i32,
    makes: Vec<Regex>,
    model_re: Regex,
    homebuilt_re: Option<Regex>,
    pub variants: Vec<Variant>,
}
pub struct Taxonomy {
    pub classes: Vec<RawClass>,
    pub families: Vec<Family>,
}

#[derive(Debug, PartialEq)]
pub enum MatchResult {
    Matched { family: String, variant: Option<String>, class: String },
    Ambiguous(Vec<String>),
    NoMatch,
}

pub fn norm_make(s: &str) -> String {
    let up: String = s
        .to_uppercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    up.split_whitespace().collect::<Vec<_>>().join(" ")
}
pub fn norm_model(s: &str) -> String {
    s.to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

impl Taxonomy {
    pub fn from_toml(text: &str) -> Result<Self> {
        let raw: RawFile = toml::from_str(text).context("parse taxonomy toml")?;
        let mut make_rx: HashMap<String, Vec<Regex>> = HashMap::new();
        for (k, v) in &raw.makes {
            let rs = v
                .iter()
                .map(|r| Regex::new(r).with_context(|| format!("make {k}: {r}")))
                .collect::<Result<Vec<_>>>()?;
            make_rx.insert(k.clone(), rs);
        }
        let class_ids: Vec<&str> = raw.classes.iter().map(|c| c.id.as_str()).collect();
        let mut families = Vec::new();
        for f in raw.families {
            if !class_ids.contains(&f.class.as_str()) {
                bail!("family {}: unknown class {}", f.id, f.class);
            }
            let mut makes = Vec::new();
            for m in &f.makes {
                makes.extend(make_rx.get(m).with_context(|| format!("family {}: unknown make key {m}", f.id))?.iter().cloned());
            }
            let variants = f
                .variant
                .into_iter()
                .map(|v| {
                    Ok(Variant {
                        re: Regex::new(&v.model_re).with_context(|| format!("{}/{}", f.id, v.id))?,
                        id: v.id,
                        class: v.class,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            families.push(Family {
                model_re: Regex::new(&f.model_re).with_context(|| format!("family {}", f.id))?,
                homebuilt_re: f.homebuilt_model_re.as_deref().map(Regex::new).transpose()?,
                id: f.id,
                class: f.class,
                priority: f.priority,
                makes,
                variants,
            });
        }
        Ok(Self { classes: raw.classes, families })
    }

    fn candidates(&self, make: &str, model: &str, homebuilt: bool) -> Vec<&Family> {
        self.families
            .iter()
            .filter(|f| {
                (f.makes.iter().any(|r| r.is_match(make)) && f.model_re.is_match(model))
                    || (homebuilt && f.homebuilt_re.as_ref().is_some_and(|r| r.is_match(model)))
            })
            .collect()
    }

    pub fn classify(&self, make_raw: &str, model_raw: &str, series_raw: &str, homebuilt: bool) -> MatchResult {
        let make = norm_make(make_raw);
        let model = norm_model(model_raw);
        let series = norm_model(series_raw);
        let combo = if !series.is_empty() && !model.ends_with(&series) { format!("{model}{series}") } else { model.clone() };
        let mut keys = vec![model.as_str()];
        if combo != model {
            keys.push(combo.as_str());
        }
        for key in keys {
            let c = self.candidates(&make, key, homebuilt);
            if c.is_empty() {
                continue;
            }
            let top = c.iter().map(|f| f.priority).max().unwrap();
            let best: Vec<&&Family> = c.iter().filter(|f| f.priority == top).collect();
            if best.len() > 1 {
                return MatchResult::Ambiguous(best.iter().map(|f| f.id.clone()).collect());
            }
            let fam = best[0];
            let mut vkeys = Vec::new();
            if combo != model {
                vkeys.push(combo.as_str());
            }
            vkeys.push(model.as_str());
            for vk in vkeys {
                if let Some(v) = fam.variants.iter().find(|v| v.re.is_match(vk)) {
                    return MatchResult::Matched {
                        family: fam.id.clone(),
                        variant: Some(v.id.clone()),
                        class: v.class.clone().unwrap_or_else(|| fam.class.clone()),
                    };
                }
            }
            return MatchResult::Matched { family: fam.id.clone(), variant: None, class: fam.class.clone() };
        }
        MatchResult::NoMatch
    }
}
