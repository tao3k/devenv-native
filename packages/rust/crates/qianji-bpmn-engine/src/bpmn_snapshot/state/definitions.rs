use super::{
    BpmnCategorySnapshot, BpmnCategoryValueSnapshot, BpmnCorrelationPropertySnapshot,
    BpmnCorrelationRetrievalExpressionSnapshot, BpmnEndPointSnapshot, BpmnErrorSnapshot,
    BpmnEscalationSnapshot, BpmnExtensionSnapshot, BpmnGlobalTaskSnapshot, BpmnImportSnapshot,
    BpmnInterfaceSnapshot, BpmnItemDefinitionSnapshot, BpmnMessageSnapshot, BpmnOperationSnapshot,
    BpmnPartnerEntitySnapshot, BpmnPartnerRoleSnapshot, BpmnRelationshipSnapshot,
    BpmnResourceParameterBindingSnapshot, BpmnResourceParameterSnapshot, BpmnResourceRoleSnapshot,
    BpmnResourceSnapshot, BpmnSignalSnapshot, BpmnSnapshotScanState, BpmnSourceFile, BytesStart,
    Reader, ResourceRoleOwner, Result, attribute_value, boolean_attribute_value,
    resource_role_from_event,
};

impl BpmnSnapshotScanState {
    pub(super) fn capture_message(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.message_count += 1;
        root.messages.push(BpmnMessageSnapshot {
            message_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_ref: attribute_value(source, reader, event, "itemRef")?,
        });
        Ok(())
    }

    pub(super) fn capture_end_point(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.end_point_count += 1;
        root.end_points.push(BpmnEndPointSnapshot {
            end_point_id: attribute_value(source, reader, event, "id")?,
        });
        Ok(())
    }

    pub(super) fn start_partner_entity(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.partner_entity_count += 1;
        root.partner_entities.push(BpmnPartnerEntitySnapshot {
            partner_entity_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            participant_refs: Vec::new(),
        });
        if !is_empty {
            self.current_partner_entity = root.partner_entities.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_partner_role(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.partner_role_count += 1;
        root.partner_roles.push(BpmnPartnerRoleSnapshot {
            partner_role_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            participant_refs: Vec::new(),
        });
        if !is_empty {
            self.current_partner_role = root.partner_roles.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_global_task(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.global_task_count += 1;
        root.global_tasks.push(BpmnGlobalTaskSnapshot {
            task_kind: tag.to_string(),
            task_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            implementation: attribute_value(source, reader, event, "implementation")?,
            script_language: attribute_value(source, reader, event, "scriptLanguage")?,
            script: None,
            supported_interface_refs: Vec::new(),
            io_specification_count: 0,
            io_specifications: Vec::new(),
            io_binding_count: 0,
            io_bindings: Vec::new(),
            resource_role_count: 0,
            resource_roles: Vec::new(),
        });
        if !is_empty {
            self.current_global_task = root.global_tasks.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_global_task_resource_role(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(task_index) = self.current_global_task else {
            return Ok(());
        };
        let role = resource_role_from_event(source, reader, event, tag)?;
        let Some(task) = self.current_global_task_mut() else {
            return Ok(());
        };
        task.resource_role_count += 1;
        task.resource_roles.push(role);
        if !is_empty {
            let role_index = task.resource_roles.len().saturating_sub(1);
            self.current_resource_role =
                Some((ResourceRoleOwner::GlobalTask(task_index), role_index));
        }
        Ok(())
    }

    pub(super) fn start_interface(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.interface_count += 1;
        root.interfaces.push(BpmnInterfaceSnapshot {
            interface_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            implementation_ref: attribute_value(source, reader, event, "implementationRef")?,
            operations: Vec::new(),
        });
        if !is_empty {
            self.current_interface = root.interfaces.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_operation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(interface_index) = self.current_interface else {
            return Ok(());
        };
        let operation = BpmnOperationSnapshot {
            operation_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            implementation_ref: attribute_value(source, reader, event, "implementationRef")?,
            in_message_ref: None,
            out_message_ref: None,
            error_refs: Vec::new(),
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(interface) = root.interfaces.get_mut(interface_index) else {
            return Ok(());
        };
        interface.operations.push(operation);
        if !is_empty {
            let operation_index = interface.operations.len().saturating_sub(1);
            self.current_operation = Some((interface_index, operation_index));
        }
        Ok(())
    }

    pub(super) fn start_resource(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.resource_count += 1;
        root.resources.push(BpmnResourceSnapshot {
            resource_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            resource_parameters: Vec::new(),
        });
        if !is_empty {
            self.current_resource = root.resources.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn capture_resource_parameter(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(resource_index) = self.current_resource else {
            return Ok(());
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(resource) = root.resources.get_mut(resource_index) else {
            return Ok(());
        };
        resource
            .resource_parameters
            .push(BpmnResourceParameterSnapshot {
                resource_parameter_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                type_ref: attribute_value(source, reader, event, "type")?,
                is_required: boolean_attribute_value(source, reader, event, "isRequired")?,
            });
        Ok(())
    }

    pub(super) fn start_category(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.category_count += 1;
        root.categories.push(BpmnCategorySnapshot {
            category_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            category_values: Vec::new(),
        });
        if !is_empty {
            self.current_category = root.categories.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn capture_category_value(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(category_index) = self.current_category else {
            return Ok(());
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(category) = root.categories.get_mut(category_index) else {
            return Ok(());
        };
        category.category_values.push(BpmnCategoryValueSnapshot {
            category_value_id: attribute_value(source, reader, event, "id")?,
            value: attribute_value(source, reader, event, "value")?,
        });
        Ok(())
    }

    pub(super) fn capture_item_definition(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.item_definition_count += 1;
        root.item_definitions.push(BpmnItemDefinitionSnapshot {
            item_definition_id: attribute_value(source, reader, event, "id")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
            item_kind: attribute_value(source, reader, event, "itemKind")?,
            is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
        });
        Ok(())
    }

    pub(super) fn capture_correlation_property(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.correlation_property_count += 1;
        root.correlation_properties
            .push(BpmnCorrelationPropertySnapshot {
                correlation_property_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                type_ref: attribute_value(source, reader, event, "type")?,
                retrieval_expressions: Vec::new(),
            });
        if !is_empty {
            self.current_correlation_property = root.correlation_properties.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_correlation_retrieval_expression(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(property_index) = self.current_correlation_property else {
            return Ok(());
        };
        let retrieval_expression = BpmnCorrelationRetrievalExpressionSnapshot {
            retrieval_expression_id: attribute_value(source, reader, event, "id")?,
            message_ref: attribute_value(source, reader, event, "messageRef")?,
            message_path: None,
        };
        if is_empty {
            self.push_correlation_retrieval_expression(property_index, retrieval_expression);
            return Ok(());
        }
        self.current_correlation_retrieval_expression =
            Some((property_index, retrieval_expression));
        Ok(())
    }

    pub(super) fn capture_error(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.error_count += 1;
        root.errors.push(BpmnErrorSnapshot {
            error_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            error_code: attribute_value(source, reader, event, "errorCode")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    pub(super) fn capture_escalation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.escalation_count += 1;
        root.escalations.push(BpmnEscalationSnapshot {
            escalation_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            escalation_code: attribute_value(source, reader, event, "escalationCode")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    pub(super) fn capture_signal(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.signal_count += 1;
        root.signals.push(BpmnSignalSnapshot {
            signal_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    pub(super) fn capture_import(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.import_count += 1;
        root.imports.push(BpmnImportSnapshot {
            namespace: attribute_value(source, reader, event, "namespace")?,
            location: attribute_value(source, reader, event, "location")?,
            import_type: attribute_value(source, reader, event, "importType")?,
        });
        Ok(())
    }

    pub(super) fn start_extension(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.extension_count += 1;
        root.extensions.push(BpmnExtensionSnapshot {
            definition: attribute_value(source, reader, event, "definition")?,
            must_understand: boolean_attribute_value(source, reader, event, "mustUnderstand")?
                .unwrap_or(false),
            documentation: Vec::new(),
        });
        if !is_empty {
            self.current_extension = root.extensions.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_extension_documentation(&mut self, is_empty: bool) {
        if is_empty {
            return;
        }
        let Some(extension_index) = self.current_extension else {
            return;
        };
        self.current_extension_documentation = Some((extension_index, String::new()));
    }

    pub(super) fn finish_extension_documentation(&mut self) {
        let Some((extension_index, documentation)) = self.current_extension_documentation.take()
        else {
            return;
        };
        let documentation = documentation.trim();
        if documentation.is_empty() {
            return;
        }
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(extension) = root.extensions.get_mut(extension_index) else {
            return;
        };
        extension.documentation.push(documentation.to_string());
    }

    pub(super) fn finish_correlation_retrieval_expression(&mut self) {
        let Some((property_index, retrieval_expression)) =
            self.current_correlation_retrieval_expression.take()
        else {
            return;
        };
        self.push_correlation_retrieval_expression(property_index, retrieval_expression);
    }

    pub(super) fn push_correlation_retrieval_expression(
        &mut self,
        property_index: usize,
        retrieval_expression: BpmnCorrelationRetrievalExpressionSnapshot,
    ) {
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(property) = root.correlation_properties.get_mut(property_index) else {
            return;
        };
        property.retrieval_expressions.push(retrieval_expression);
    }

    pub(super) fn current_global_task_mut(&mut self) -> Option<&mut BpmnGlobalTaskSnapshot> {
        let global_task_index = self.current_global_task?;
        self.root.as_mut()?.global_tasks.get_mut(global_task_index)
    }

    pub(super) fn resource_role_mut(
        &mut self,
        owner: ResourceRoleOwner,
        role_index: usize,
    ) -> Option<&mut BpmnResourceRoleSnapshot> {
        match owner {
            ResourceRoleOwner::Process(process_index) => self
                .processes
                .get_mut(process_index)?
                .resource_roles
                .get_mut(role_index),
            ResourceRoleOwner::GlobalTask(task_index) => self
                .root
                .as_mut()?
                .global_tasks
                .get_mut(task_index)?
                .resource_roles
                .get_mut(role_index),
        }
    }

    pub(super) fn current_resource_assignment_expression_mut(
        &mut self,
    ) -> Option<&mut BpmnResourceRoleSnapshot> {
        let (owner, role_index) = self.current_resource_assignment_expression?;
        self.resource_role_mut(owner, role_index)
    }

    pub(super) fn current_resource_parameter_binding_mut(
        &mut self,
    ) -> Option<&mut BpmnResourceParameterBindingSnapshot> {
        let (owner, role_index, binding_index) = self.current_resource_parameter_binding?;
        self.resource_role_mut(owner, role_index)?
            .parameter_bindings
            .get_mut(binding_index)
    }

    pub(super) fn current_operation_mut(&mut self) -> Option<&mut BpmnOperationSnapshot> {
        let (interface_index, operation_index) = self.current_operation?;
        self.root
            .as_mut()?
            .interfaces
            .get_mut(interface_index)?
            .operations
            .get_mut(operation_index)
    }

    pub(super) fn current_relationship_mut(&mut self) -> Option<&mut BpmnRelationshipSnapshot> {
        let relationship_index = self.current_relationship?;
        self.root
            .as_mut()?
            .relationships
            .get_mut(relationship_index)
    }
}
