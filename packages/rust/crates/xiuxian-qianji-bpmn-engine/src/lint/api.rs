//! Internal lint api seam over BPMN and DMN lint implementations.

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::dmn_model_api::DmnSourceFile;
use crate::lint_api::LintReport;

pub(crate) fn lint_bpmn_source_impl(source: &BpmnSourceFile) -> LintReport {
    super::bpmn::lint_bpmn_source_impl(source)
}

pub(crate) fn lint_dmn_source_impl(source: &DmnSourceFile) -> LintReport {
    super::dmn::lint_dmn_source_impl(source)
}
