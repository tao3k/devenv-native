use crate::dmn_model_api::DmnDocumentSnapshot;

pub(super) fn snapshot_has_decision_service(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| snapshot.root.decision_service_count > 0)
}

pub(super) fn snapshot_import_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.import_count.max(1))
}

pub(super) fn snapshot_has_only_input_data(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count > 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_item_definition(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count > 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_knowledge_source(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count > 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_business_knowledge_model(
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count > 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_organization_unit(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count > 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_performance_indicator(
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count > 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_text_annotation(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count > 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_association(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count > 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_element_collection(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count > 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_group(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count > 0
            && snapshot.root.dmndi_count == 0
    })
}

pub(super) fn snapshot_has_only_dmndi(snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.root.item_definition_count == 0
            && snapshot.root.input_data_count == 0
            && snapshot.root.knowledge_source_count == 0
            && snapshot.root.business_knowledge_model_count == 0
            && snapshot.root.decision_service_count == 0
            && snapshot.root.organization_unit_count == 0
            && snapshot.root.performance_indicator_count == 0
            && snapshot.root.text_annotation_count == 0
            && snapshot.root.association_count == 0
            && snapshot.root.element_collection_count == 0
            && snapshot.root.group_count == 0
            && snapshot.root.dmndi_count > 0
    })
}
