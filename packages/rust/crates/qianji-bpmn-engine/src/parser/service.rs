use super::import::import_bpmn_source;
use super::normalize::normalize_package;
use super::validate::validate_raw_package;
use crate::bpmn_parse_api::{BpmnBundleSnapshot, BpmnParseOptions};
use crate::dmn_parse_api::parse_dmn_decision;
use crate::error::{BpmnEngineError, Result};
use crate::ir_package_api::BpmnPackage;

pub(crate) fn parse_bpmn_bundle_impl(
    snapshot: &BpmnBundleSnapshot,
    options: &BpmnParseOptions,
) -> Result<BpmnPackage> {
    if snapshot.bpmn_sources.len() != 1 {
        return Err(BpmnEngineError::UnsupportedSourceBundle {
            count: snapshot.bpmn_sources.len(),
        });
    }
    if options.validate_schema {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_schema_validation",
        });
    }

    let raw = import_bpmn_source(&snapshot.bpmn_sources[0])?;
    validate_raw_package(&raw)?;
    let package = normalize_package(raw)?;
    let dmn_decisions = snapshot
        .dmn_sources
        .iter()
        .map(parse_dmn_decision)
        .collect::<Result<Vec<_>>>()?;
    if dmn_decisions.is_empty() {
        Ok(package)
    } else {
        Ok(package.with_dmn_decisions(dmn_decisions))
    }
}
