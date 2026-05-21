//! Semantic lifecycle planning and writeback interface.

mod entry;

pub(crate) use entry::{
    SemanticLifecyclePlanReport, apply_semantic_lifecycle_plan, semantic_lifecycle_plan_report,
};
