//! What–Why Chain Analysis (WWCA).
//!
//! Pipeline: `Dict::from_toml` -> `build::build` per involvement -> `metrics::*`.
//! Normative description: ANALYSIS_METHOD_SPEC.md.
pub mod build;
pub mod dict;
pub mod metrics;
pub mod model;
pub mod text;

pub use build::{build, BuildOpts, Graph, Prov};
pub use dict::{Dict, Source, Tier};
pub use metrics::{Ctx, Depth, Query};
pub use model::{FindRec, Injury, Involvement, OccRec, Role, RoleFilter};
pub use text::{normalize_find08, normalize_occ08, split_find08, split_occ08};
