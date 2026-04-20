//! Lint contracts for BPMN and DMN sources.

mod bpmn;
mod dmn;
mod model;

pub use bpmn::lint_bpmn_source;
pub use dmn::lint_dmn_source;
pub use model::{LintDomain, LintIssue, LintReport, LintSeverity};
