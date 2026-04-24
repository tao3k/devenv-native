mod context;
mod facts;
mod link;
mod text;

pub(super) use context::DiagnosticContext;
pub(in crate::lint) use context::LocalTargetFragmentResolution;
pub(in crate::lint) use context::LocalTargetResolution;
pub(in crate::lint) use facts::{
    DiagnosticFacts, DynamicDiagnosticText, LocalTargetFragmentViolation,
    LocalTargetScopeViolation, LocalTargetTransientViolation,
};
pub(super) use text::{code_string, markdown_lint_issue_codes, markdown_lint_rule_keys};
