//! What–Why Chain Analysis (WWCA).
//!
//! Pipeline: `Dict::from_toml` -> `build::build` per involvement -> `metrics::*`.
//! Normative description: ANALYSIS_METHOD_SPEC.md.
pub mod build;
pub mod dict;
pub mod metrics;
pub mod model;

pub use build::{build, BuildOpts, Graph, Prov};
pub use dict::{Dict, Source, Tier};
pub use metrics::{Ctx, Depth, Query};
pub use model::{FindRec, Injury, Involvement, OccRec, Role, RoleFilter};
