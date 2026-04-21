use crate::dmn_model_api::DmnDocumentSnapshot;

pub(super) fn snapshot_item_definition_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.item_definition_count.max(1))
}

pub(super) fn snapshot_input_data_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.input_data_count.max(1))
}

pub(super) fn snapshot_knowledge_source_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.knowledge_source_count.max(1))
}

pub(super) fn snapshot_business_knowledge_model_count(
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot.map_or(1, |snapshot| {
        snapshot.root.business_knowledge_model_count.max(1)
    })
}

pub(super) fn snapshot_organization_unit_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.organization_unit_count.max(1))
}

pub(super) fn snapshot_performance_indicator_count(
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot.map_or(1, |snapshot| {
        snapshot.root.performance_indicator_count.max(1)
    })
}

pub(super) fn snapshot_text_annotation_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.text_annotation_count.max(1))
}

pub(super) fn snapshot_association_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.association_count.max(1))
}

pub(super) fn snapshot_element_collection_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.element_collection_count.max(1))
}

pub(super) fn snapshot_group_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.group_count.max(1))
}

pub(super) fn snapshot_dmndi_count(snapshot: Option<&DmnDocumentSnapshot>) -> usize {
    snapshot.map_or(1, |snapshot| snapshot.root.dmndi_count.max(1))
}
