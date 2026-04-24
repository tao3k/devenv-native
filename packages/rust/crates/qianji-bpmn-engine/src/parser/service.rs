use super::import::import_bpmn_source;
use super::normalize::normalize_package;
use super::validate::validate_raw_package;
use crate::bpmn_parse_api::{BpmnBundleSnapshot, BpmnParseOptions};
use crate::dmn_model_api::{
    DmnBusinessKnowledgeModelDefinition, DmnDecisionServiceDefinition, DmnImportDefinition,
    DmnInputDataDefinition, DmnSourceDefinition,
};
use crate::dmn_parse_api::parse_dmn_decisions;
use crate::dmn_snapshot_api::snapshot_dmn_source;
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
    let mut dmn_source_definitions = Vec::new();
    let mut dmn_imports = Vec::new();
    let mut dmn_decisions = Vec::new();
    let mut dmn_input_data = Vec::new();
    let mut dmn_business_knowledge_models = Vec::new();
    let mut dmn_decision_services = Vec::new();
    for source in &snapshot.dmn_sources {
        let snapshot = snapshot_dmn_source(source)?;
        dmn_source_definitions.push(DmnSourceDefinition::from_root_snapshot(
            &source.source_id,
            &snapshot.root,
        ));
        dmn_imports.extend(
            snapshot.root.imports.iter().map(|dmn_import| {
                DmnImportDefinition::from_snapshot(&source.source_id, dmn_import)
            }),
        );
        // Top-level imports keep the bundled source metadata-only until
        // cross-document DMN execution has an explicit contract.
        if snapshot.root.import_count > 0 {
            continue;
        }
        dmn_decisions.extend(parse_dmn_decisions(source)?);
        dmn_input_data.extend(snapshot.root.input_data.iter().map(|input_data| {
            DmnInputDataDefinition::from_snapshot(&source.source_id, input_data)
        }));
        dmn_business_knowledge_models.extend(snapshot.root.business_knowledge_models.iter().map(
            |business_knowledge_model| {
                DmnBusinessKnowledgeModelDefinition::from_snapshot(
                    &source.source_id,
                    business_knowledge_model,
                )
            },
        ));
        dmn_decision_services.extend(snapshot.root.decision_services.iter().map(
            |decision_service| {
                DmnDecisionServiceDefinition::from_snapshot(&source.source_id, decision_service)
            },
        ));
    }

    let package = if dmn_decisions.is_empty() {
        package
    } else {
        package.with_dmn_decisions(dmn_decisions)
    };
    let package = if dmn_source_definitions.is_empty() {
        package
    } else {
        package.with_dmn_source_definitions(dmn_source_definitions)
    };
    let package = if dmn_imports.is_empty() {
        package
    } else {
        package.with_dmn_imports(dmn_imports)
    };
    let package = if dmn_input_data.is_empty() {
        package
    } else {
        package.with_dmn_input_data(dmn_input_data)
    };
    let package = if dmn_business_knowledge_models.is_empty() {
        package
    } else {
        package.with_dmn_business_knowledge_models(dmn_business_knowledge_models)
    };
    if dmn_decision_services.is_empty() {
        Ok(package)
    } else {
        Ok(package.with_dmn_decision_services(dmn_decision_services))
    }
}
