use super::{
    BpmnCorrelationPropertyBindingSnapshot, BpmnCorrelationSubscriptionSnapshot,
    BpmnFlowElementMetadataSnapshot, BpmnProcessPropertySnapshot, BpmnProcessSnapshot,
    BpmnRelationshipSnapshot, BpmnResourceParameterBindingSnapshot, BpmnSnapshotScanState,
    BpmnSourceFile, BytesStart, Reader, ResourceRoleOwner, Result, attribute_value,
    boolean_attribute_value, resource_role_from_event,
};

impl BpmnSnapshotScanState {
    pub(super) fn start_relationship(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.relationship_count += 1;
        root.relationships.push(BpmnRelationshipSnapshot {
            relationship_id: attribute_value(source, reader, event, "id")?,
            relationship_type: attribute_value(source, reader, event, "type")?,
            direction: attribute_value(source, reader, event, "direction")?,
            source_refs: Vec::new(),
            target_refs: Vec::new(),
        });
        if !is_empty {
            self.current_relationship = root.relationships.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_process(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let process = BpmnProcessSnapshot {
            process_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            process_type: attribute_value(source, reader, event, "processType")?,
            is_closed: boolean_attribute_value(source, reader, event, "isClosed")?,
            is_executable: boolean_attribute_value(source, reader, event, "isExecutable")?,
            definitional_collaboration_ref: attribute_value(
                source,
                reader,
                event,
                "definitionalCollaborationRef",
            )?,
            support_count: 0,
            supports: Vec::new(),
            property_count: 0,
            properties: Vec::new(),
            correlation_subscription_count: 0,
            correlation_subscriptions: Vec::new(),
            resource_role_count: 0,
            resource_roles: Vec::new(),
            flow_element_metadata_count: 0,
            flow_element_metadata: Vec::new(),
            lane_set_count: 0,
            lane_sets: Vec::new(),
            data_object_count: 0,
            data_objects: Vec::new(),
            data_object_reference_count: 0,
            data_object_references: Vec::new(),
            data_store_reference_count: 0,
            data_store_references: Vec::new(),
            io_specification_count: 0,
            io_specifications: Vec::new(),
            io_binding_count: 0,
            io_bindings: Vec::new(),
            data_input_association_count: 0,
            data_input_associations: Vec::new(),
            data_output_association_count: 0,
            data_output_associations: Vec::new(),
            association_count: 0,
            associations: Vec::new(),
            group_count: 0,
            groups: Vec::new(),
            text_annotation_count: 0,
            text_annotations: Vec::new(),
        };
        self.processes.push(process);
        if let Some(root) = self.root.as_mut() {
            root.process_count += 1;
        }
        if !is_empty {
            self.current_process = self.processes.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_flow_element_metadata(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_flow_element_metadata();
        if is_empty {
            return Ok(());
        }
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        self.current_flow_element_metadata = Some((
            process_index,
            BpmnFlowElementMetadataSnapshot {
                element_kind: tag.to_string(),
                element_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                has_auditing: false,
                auditing_id: None,
                has_monitoring: false,
                monitoring_id: None,
                category_value_refs: Vec::new(),
            },
        ));
        Ok(())
    }

    pub(super) fn attach_flow_element_auditing(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((_, metadata)) = self.current_flow_element_metadata.as_mut() else {
            return Ok(());
        };
        metadata.has_auditing = true;
        metadata.auditing_id = attribute_value(source, reader, event, "id")?;
        Ok(())
    }

    pub(super) fn attach_flow_element_monitoring(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((_, metadata)) = self.current_flow_element_metadata.as_mut() else {
            return Ok(());
        };
        metadata.has_monitoring = true;
        metadata.monitoring_id = attribute_value(source, reader, event, "id")?;
        Ok(())
    }

    pub(super) fn capture_process_property(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let property = BpmnProcessPropertySnapshot {
            property_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
        };
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.property_count += 1;
        process.properties.push(property);
        Ok(())
    }

    pub(super) fn start_process_resource_role(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let role = resource_role_from_event(source, reader, event, tag)?;
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.resource_role_count += 1;
        process.resource_roles.push(role);
        if !is_empty {
            let role_index = process.resource_roles.len().saturating_sub(1);
            self.current_resource_role =
                Some((ResourceRoleOwner::Process(process_index), role_index));
        }
        Ok(())
    }

    pub(super) fn start_resource_parameter_binding(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((owner, role_index)) = self.current_resource_role else {
            return Ok(());
        };
        let binding = BpmnResourceParameterBindingSnapshot {
            binding_id: attribute_value(source, reader, event, "id")?,
            parameter_ref: attribute_value(source, reader, event, "parameterRef")?,
            expression: None,
            expression_language: None,
            expression_evaluates_to_type_ref: None,
        };
        let Some(role) = self.resource_role_mut(owner, role_index) else {
            return Ok(());
        };
        role.parameter_bindings.push(binding);
        if !is_empty {
            let binding_index = role.parameter_bindings.len().saturating_sub(1);
            self.current_resource_parameter_binding = Some((owner, role_index, binding_index));
        }
        Ok(())
    }

    pub(super) fn start_resource_assignment_expression(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((owner, role_index)) = self.current_resource_role else {
            return Ok(());
        };
        let assignment_expression_id = attribute_value(source, reader, event, "id")?;
        let Some(role) = self.resource_role_mut(owner, role_index) else {
            return Ok(());
        };
        role.assignment_expression_id = assignment_expression_id;
        if !is_empty {
            self.current_resource_assignment_expression = Some((owner, role_index));
        }
        Ok(())
    }

    pub(super) fn attach_resource_assignment_expression_metadata(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let language = attribute_value(source, reader, event, "language")?;
        let evaluates_to_type_ref = attribute_value(source, reader, event, "evaluatesToTypeRef")?;
        let Some(role) = self.current_resource_assignment_expression_mut() else {
            return Ok(());
        };
        role.assignment_expression_language = language;
        role.assignment_expression_evaluates_to_type_ref = evaluates_to_type_ref;
        Ok(())
    }

    pub(super) fn attach_resource_parameter_binding_expression_metadata(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let language = attribute_value(source, reader, event, "language")?;
        let evaluates_to_type_ref = attribute_value(source, reader, event, "evaluatesToTypeRef")?;
        let Some(binding) = self.current_resource_parameter_binding_mut() else {
            return Ok(());
        };
        binding.expression_language = language;
        binding.expression_evaluates_to_type_ref = evaluates_to_type_ref;
        Ok(())
    }

    pub(super) fn start_correlation_subscription(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let subscription = BpmnCorrelationSubscriptionSnapshot {
            subscription_id: attribute_value(source, reader, event, "id")?,
            correlation_key_ref: attribute_value(source, reader, event, "correlationKeyRef")?,
            bindings: Vec::new(),
        };
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.correlation_subscription_count += 1;
        process.correlation_subscriptions.push(subscription);
        if !is_empty {
            let subscription_index = process.correlation_subscriptions.len().saturating_sub(1);
            self.current_correlation_subscription = Some((process_index, subscription_index));
        }
        Ok(())
    }

    pub(super) fn start_correlation_property_binding(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((process_index, subscription_index)) = self.current_correlation_subscription
        else {
            return Ok(());
        };
        let binding = BpmnCorrelationPropertyBindingSnapshot {
            binding_id: attribute_value(source, reader, event, "id")?,
            correlation_property_ref: attribute_value(
                source,
                reader,
                event,
                "correlationPropertyRef",
            )?,
            data_path: None,
            data_path_language: None,
            data_path_evaluates_to_type_ref: None,
        };
        let Some(subscription) =
            self.correlation_subscription_mut(process_index, subscription_index)
        else {
            return Ok(());
        };
        subscription.bindings.push(binding);
        if !is_empty {
            let binding_index = subscription.bindings.len().saturating_sub(1);
            self.current_correlation_property_binding =
                Some((process_index, subscription_index, binding_index));
        }
        Ok(())
    }

    pub(super) fn attach_correlation_binding_data_path_metadata(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let language = attribute_value(source, reader, event, "language")?;
        let evaluates_to_type_ref = attribute_value(source, reader, event, "evaluatesToTypeRef")?;
        let Some(binding) = self.current_correlation_property_binding_mut() else {
            return Ok(());
        };
        binding.data_path_language = language;
        binding.data_path_evaluates_to_type_ref = evaluates_to_type_ref;
        Ok(())
    }

    pub(super) fn finish_flow_element_metadata(&mut self) {
        self.collecting_flow_element_category_value_ref = false;
        let Some((process_index, metadata)) = self.current_flow_element_metadata.take() else {
            return;
        };
        if !metadata.has_auditing
            && !metadata.has_monitoring
            && metadata.category_value_refs.is_empty()
        {
            return;
        }
        let Some(process) = self.processes.get_mut(process_index) else {
            return;
        };
        process.flow_element_metadata_count += 1;
        process.flow_element_metadata.push(metadata);
    }

    pub(super) fn current_process_mut(&mut self) -> Option<&mut BpmnProcessSnapshot> {
        self.current_process
            .and_then(|index| self.processes.get_mut(index))
    }

    pub(super) fn correlation_subscription_mut(
        &mut self,
        process_index: usize,
        subscription_index: usize,
    ) -> Option<&mut BpmnCorrelationSubscriptionSnapshot> {
        self.processes
            .get_mut(process_index)?
            .correlation_subscriptions
            .get_mut(subscription_index)
    }

    pub(super) fn current_correlation_property_binding_mut(
        &mut self,
    ) -> Option<&mut BpmnCorrelationPropertyBindingSnapshot> {
        let (process_index, subscription_index, binding_index) =
            self.current_correlation_property_binding?;
        self.correlation_subscription_mut(process_index, subscription_index)?
            .bindings
            .get_mut(binding_index)
    }
}
