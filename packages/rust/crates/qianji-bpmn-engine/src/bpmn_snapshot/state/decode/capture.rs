use crate::bpmn_snapshot::state::{
    BpmnBoundsSnapshot, BpmnDataAssociationExpressionSnapshot, BpmnDataInputOutputSnapshot,
    BpmnDataStateSnapshot, BpmnFontSnapshot, BpmnIoBindingSnapshot, BpmnLabelSnapshot,
    BpmnResourceRoleSnapshot, BpmnRootSnapshot, BpmnSourceFile, BpmnWaypointSnapshot, BytesStart,
    Reader, Result, attribute_value, boolean_attribute_value, bpmn_model_namespace, local_name,
};

pub(in crate::bpmn_snapshot::state) fn label_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnLabelSnapshot> {
    Ok(BpmnLabelSnapshot {
        label_id: attribute_value(source, reader, event, "id")?,
        label_style: attribute_value(source, reader, event, "labelStyle")?,
        bounds: None,
    })
}

pub(in crate::bpmn_snapshot::state) fn bounds_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnBoundsSnapshot> {
    Ok(BpmnBoundsSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
        width: attribute_value(source, reader, event, "width")?,
        height: attribute_value(source, reader, event, "height")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn waypoint_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnWaypointSnapshot> {
    Ok(BpmnWaypointSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn font_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnFontSnapshot> {
    Ok(BpmnFontSnapshot {
        name: attribute_value(source, reader, event, "name")?,
        size: attribute_value(source, reader, event, "size")?,
        is_bold: boolean_attribute_value(source, reader, event, "isBold")?,
        is_italic: boolean_attribute_value(source, reader, event, "isItalic")?,
        is_underline: boolean_attribute_value(source, reader, event, "isUnderline")?,
        is_strike_through: boolean_attribute_value(source, reader, event, "isStrikeThrough")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn root_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnRootSnapshot> {
    let event_name = event.name();
    Ok(BpmnRootSnapshot {
        element_name: local_name(event_name.as_ref()).to_string(),
        definitions_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        target_namespace: attribute_value(source, reader, event, "targetNamespace")?,
        model_namespace_uri: bpmn_model_namespace(source, reader, event)?,
        import_count: 0,
        imports: Vec::new(),
        extension_count: 0,
        extensions: Vec::new(),
        relationship_count: 0,
        relationships: Vec::new(),
        diagram_count: 0,
        diagrams: Vec::new(),
        collaboration_count: 0,
        process_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        message_count: 0,
        messages: Vec::new(),
        interface_count: 0,
        interfaces: Vec::new(),
        end_point_count: 0,
        end_points: Vec::new(),
        resource_count: 0,
        resources: Vec::new(),
        category_count: 0,
        categories: Vec::new(),
        correlation_property_count: 0,
        correlation_properties: Vec::new(),
        error_count: 0,
        errors: Vec::new(),
        escalation_count: 0,
        escalations: Vec::new(),
        signal_count: 0,
        signals: Vec::new(),
        data_store_count: 0,
        data_stores: Vec::new(),
        partner_entity_count: 0,
        partner_entities: Vec::new(),
        partner_role_count: 0,
        partner_roles: Vec::new(),
        global_task_count: 0,
        global_tasks: Vec::new(),
    })
}

pub(in crate::bpmn_snapshot::state) fn data_input_output_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnDataInputOutputSnapshot> {
    Ok(BpmnDataInputOutputSnapshot {
        data_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
        is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
        data_state: None,
    })
}

pub(in crate::bpmn_snapshot::state) fn data_state_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnDataStateSnapshot> {
    Ok(BpmnDataStateSnapshot {
        data_state_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn io_binding_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnIoBindingSnapshot> {
    Ok(BpmnIoBindingSnapshot {
        binding_id: attribute_value(source, reader, event, "id")?,
        operation_ref: attribute_value(source, reader, event, "operationRef")?,
        input_data_ref: attribute_value(source, reader, event, "inputDataRef")?,
        output_data_ref: attribute_value(source, reader, event, "outputDataRef")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn data_association_expression_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnDataAssociationExpressionSnapshot> {
    Ok(BpmnDataAssociationExpressionSnapshot {
        expression_id: attribute_value(source, reader, event, "id")?,
        body: None,
        language: attribute_value(source, reader, event, "language")?,
        evaluates_to_type_ref: attribute_value(source, reader, event, "evaluatesToTypeRef")?,
    })
}

pub(in crate::bpmn_snapshot::state) fn resource_role_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
) -> Result<BpmnResourceRoleSnapshot> {
    Ok(BpmnResourceRoleSnapshot {
        role_kind: tag.to_string(),
        role_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        resource_ref: None,
        assignment_expression_id: None,
        assignment_expression: None,
        assignment_expression_language: None,
        assignment_expression_evaluates_to_type_ref: None,
        parameter_bindings: Vec::new(),
    })
}
