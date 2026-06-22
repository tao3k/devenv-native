use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{DmnSourceFile, snapshot_dmn_source};

#[test]
fn dmn_snapshot_counts_top_level_item_definitions_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-item-definition-20191111.dmn",
    ))
    .must("item-definition-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 1);
    assert_eq!(snapshot.root.item_definitions.len(), 1);
    let item_definition = &snapshot.root.item_definitions[0];
    assert_eq!(
        item_definition.item_definition_id.as_deref(),
        Some("ItemDefinition_loan_offer")
    );
    assert_eq!(item_definition.name.as_deref(), Some("tLoanOffer"));
    assert_eq!(item_definition.type_ref, None);
    assert_eq!(item_definition.is_collection, Some(false));
    assert_eq!(item_definition.item_components.len(), 1);
    let item_component = &item_definition.item_components[0];
    assert_eq!(
        item_component.item_component_id.as_deref(),
        Some("ItemDefinition_loan_offer_amount")
    );
    assert_eq!(item_component.name.as_deref(), Some("amount"));
    assert_eq!(item_component.type_ref.as_deref(), Some("number"));
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_top_level_item_definition_typeref_metadata() {
    let source = DmnSourceFile::new(
        "inline-item-definition-typeref.dmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<semantic:definitions
    xmlns:semantic="https://www.omg.org/spec/DMN/20191111/MODEL/"
    id="Definitions_inline_item_definition_typeref"
    name="Inline Item Definition TypeRef"
    namespace="https://qianji.dev/dmn/inline-item-definition-typeref">
    <semantic:itemDefinition
        id="ItemDefinition_inline_code"
        name="tInlineCode"
        typeRef="string" />
</semantic:definitions>"#,
    );

    let snapshot = snapshot_dmn_source(&source)
        .must("inline item-definition source should still produce a document snapshot");

    assert_eq!(snapshot.root.item_definition_count, 1);
    assert_eq!(snapshot.root.item_definitions.len(), 1);
    let item_definition = &snapshot.root.item_definitions[0];
    assert_eq!(
        item_definition.item_definition_id.as_deref(),
        Some("ItemDefinition_inline_code")
    );
    assert_eq!(item_definition.name.as_deref(), Some("tInlineCode"));
    assert_eq!(item_definition.type_ref.as_deref(), Some("string"));
    assert_eq!(item_definition.is_collection, None);
    assert!(item_definition.item_components.is_empty());
}
