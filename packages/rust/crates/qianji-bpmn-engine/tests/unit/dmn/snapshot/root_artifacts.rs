use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{DmnSourceFile, snapshot_dmn_source};

#[test]
fn dmn_snapshot_counts_top_level_input_data_artifacts_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("metadata-only-input-data-20191111.dmn"))
        .must("input-data-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 1);
    assert_eq!(snapshot.root.input_data.len(), 1);
    let input_data = &snapshot.root.input_data[0];
    assert_eq!(
        input_data.input_data_id.as_deref(),
        Some("InputData_applicant")
    );
    assert_eq!(input_data.name.as_deref(), Some("Applicant Input"));
    assert_eq!(input_data.variable, None);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_top_level_input_data_variable_metadata() {
    let source = DmnSourceFile::new(
        "inline-input-data-variable.dmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions
  id="Definitions_inline_input_data_variable"
  name="Inline Input Data Variable"
  namespace="https://qianji.dev/dmn/inline-input-data-variable"
  xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/">
  <inputData id="InputData_income" name="Applicant Income">
    <variable id="Variable_income" name="income" typeRef="number" />
  </inputData>
</definitions>"#,
    );

    let snapshot = snapshot_dmn_source(&source)
        .must("input-data variable source should still produce a document snapshot");

    assert_eq!(snapshot.root.input_data_count, 1);
    assert_eq!(snapshot.root.input_data.len(), 1);
    let input_data = &snapshot.root.input_data[0];
    assert_eq!(
        input_data.input_data_id.as_deref(),
        Some("InputData_income")
    );
    assert_eq!(input_data.name.as_deref(), Some("Applicant Income"));
    assert_eq!(
        input_data.variable.as_ref().map(|variable| (
            variable.variable_id.as_deref(),
            variable.name.as_deref(),
            variable.type_ref.as_deref(),
        )),
        Some((Some("Variable_income"), Some("income"), Some("number")))
    );
}

#[test]
fn dmn_snapshot_counts_top_level_knowledge_source_artifacts_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-knowledge-source-20191111.dmn",
    ))
    .must("knowledge-source-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 1);
    assert_eq!(snapshot.root.knowledge_sources.len(), 1);
    let knowledge_source = &snapshot.root.knowledge_sources[0];
    assert_eq!(
        knowledge_source.knowledge_source_id.as_deref(),
        Some("KnowledgeSource_policy")
    );
    assert_eq!(knowledge_source.name.as_deref(), Some("Policy Authority"));
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_top_level_knowledge_source_metadata_from_non_empty_tag() {
    let source = DmnSourceFile::new(
        "inline-knowledge-source.dmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions
  id="Definitions_inline_knowledge_source"
  name="Inline Knowledge Source"
  namespace="https://qianji.dev/dmn/inline-knowledge-source"
  xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/">
  <knowledgeSource id="KnowledgeSource_regulator" name="Regulator Source"></knowledgeSource>
</definitions>"#,
    );

    let snapshot = snapshot_dmn_source(&source)
        .must("knowledge-source source should still produce a document snapshot");

    assert_eq!(snapshot.root.knowledge_source_count, 1);
    assert_eq!(
        snapshot
            .root
            .knowledge_sources
            .iter()
            .map(|knowledge_source| (
                knowledge_source.knowledge_source_id.as_deref(),
                knowledge_source.name.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![(Some("KnowledgeSource_regulator"), Some("Regulator Source"))]
    );
}

#[test]
fn dmn_snapshot_counts_top_level_business_knowledge_model_artifacts_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-business-knowledge-model-20191111.dmn",
    ))
    .must("business-knowledge-model-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 1);
    assert_eq!(snapshot.root.business_knowledge_models.len(), 1);
    let business_knowledge_model = &snapshot.root.business_knowledge_models[0];
    assert_eq!(
        business_knowledge_model
            .business_knowledge_model_id
            .as_deref(),
        Some("BKM_policy_source")
    );
    assert_eq!(
        business_knowledge_model.name.as_deref(),
        Some("Policy Source")
    );
    assert_eq!(
        business_knowledge_model.body.as_ref().map(|body| (
            body.expression_id.as_deref(),
            body.type_ref.as_deref(),
            body.text.as_deref(),
        )),
        Some((
            Some("BKM_policy_source_expression"),
            None,
            Some("\"external-policy\"")
        ))
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_top_level_business_knowledge_model_invocable_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-business-knowledge-model-invocable-20191111.dmn",
    ))
    .must("business-knowledge-model invocable source should still produce a document snapshot");

    assert_eq!(snapshot.root.business_knowledge_model_count, 1);
    assert_eq!(snapshot.root.business_knowledge_models.len(), 1);
    let business_knowledge_model = &snapshot.root.business_knowledge_models[0];
    assert_eq!(
        business_knowledge_model
            .business_knowledge_model_id
            .as_deref(),
        Some("BKM_policy_source")
    );
    assert_eq!(
        business_knowledge_model.variable.as_ref().map(|variable| (
            variable.variable_id.as_deref(),
            variable.name.as_deref(),
            variable.type_ref.as_deref(),
        )),
        Some((Some("Variable_policy"), Some("policy"), Some("string")))
    );
    assert_eq!(
        business_knowledge_model
            .encapsulated_logic
            .as_ref()
            .map(|logic| (
                logic.function_definition_id.as_deref(),
                logic.kind.as_deref(),
                logic.parameters.first().map(|parameter| (
                    parameter.parameter_id.as_deref(),
                    parameter.name.as_deref(),
                    parameter.type_ref.as_deref(),
                )),
                logic.body.as_ref().map(|body| (
                    body.expression_id.as_deref(),
                    body.type_ref.as_deref(),
                    body.text.as_deref(),
                )),
            )),
        Some((
            Some("EncapsulatedLogic_policy"),
            Some("FEEL"),
            Some((
                Some("Parameter_applicant"),
                Some("applicant"),
                Some("string")
            )),
            Some((
                Some("EncapsulatedLogic_policy_body"),
                Some("string"),
                Some("\"external-policy\""),
            )),
        ))
    );
    assert_eq!(business_knowledge_model.body, None);
}
