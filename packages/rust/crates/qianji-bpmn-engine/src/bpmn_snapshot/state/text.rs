use super::{BpmnSnapshotScanState, TextTarget};

impl BpmnSnapshotScanState {
    pub(in crate::bpmn_snapshot) fn handle_text_chunk(
        &mut self,
        text: &str,
        target: Option<TextTarget>,
    ) {
        let Some(target) = target else {
            return;
        };
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        match target {
            TextTarget::LaneFlowNode => self.push_lane_flow_node_ref(text),
            TextTarget::DataAssociationSource => self.push_data_association_source_ref(text),
            TextTarget::DataAssociationTarget => self.set_data_association_target_ref(text),
            TextTarget::DataAssociationTransformation => {
                self.append_data_association_transformation(text);
            }
            TextTarget::DataAssociationAssignmentFrom => {
                self.append_data_association_assignment_from(text);
            }
            TextTarget::DataAssociationAssignmentTo => {
                self.append_data_association_assignment_to(text);
            }
            TextTarget::CorrelationMessagePath => self.append_correlation_message_path(text),
            TextTarget::CorrelationBindingDataPath => {
                self.append_correlation_binding_data_path(text);
            }
            TextTarget::ResourceRoleResourceRef => self.set_resource_role_resource_ref(text),
            TextTarget::ResourceRoleAssignmentExpression => {
                self.append_resource_role_assignment_expression(text);
            }
            TextTarget::ResourceRoleParameterBindingExpression => {
                self.append_resource_parameter_binding_expression(text);
            }
            TextTarget::FlowElementCategoryValueRef => {
                self.push_flow_element_category_value_ref(text);
            }
            TextTarget::OperationInMessageRef => self.set_operation_in_message_ref(text),
            TextTarget::OperationOutMessageRef => self.set_operation_out_message_ref(text),
            TextTarget::OperationErrorRef => self.push_operation_error_ref(text),
            TextTarget::IoInputSetDataInputRef => self.push_io_input_set_data_input_ref(text),
            TextTarget::IoInputSetOptionalInputRef => {
                self.push_io_input_set_optional_input_ref(text);
            }
            TextTarget::IoInputSetWhileExecutingInputRef => {
                self.push_io_input_set_while_executing_input_ref(text);
            }
            TextTarget::IoInputSetOutputSetRef => self.push_io_input_set_output_set_ref(text),
            TextTarget::IoOutputSetDataOutputRef => self.push_io_output_set_data_output_ref(text),
            TextTarget::IoOutputSetOptionalOutputRef => {
                self.push_io_output_set_optional_output_ref(text);
            }
            TextTarget::IoOutputSetWhileExecutingOutputRef => {
                self.push_io_output_set_while_executing_output_ref(text);
            }
            TextTarget::IoOutputSetInputSetRef => self.push_io_output_set_input_set_ref(text),
            TextTarget::ExtensionDocumentation => self.append_extension_documentation(text),
            TextTarget::RelationshipSource => self.push_relationship_source_ref(text),
            TextTarget::RelationshipTarget => self.push_relationship_target_ref(text),
            TextTarget::ParticipantInterfaceRef => self.push_participant_interface_ref(text),
            TextTarget::ParticipantEndPointRef => self.push_participant_end_point_ref(text),
            TextTarget::PartnerEntityParticipantRef => {
                self.push_partner_entity_participant_ref(text);
            }
            TextTarget::PartnerRoleParticipantRef => self.push_partner_role_participant_ref(text),
            TextTarget::GlobalTaskSupportedInterfaceRef => {
                self.push_global_task_supported_interface_ref(text);
            }
            TextTarget::GlobalTaskScript => self.append_global_task_script(text),
            TextTarget::ProcessSupport => self.push_process_support_ref(text),
            TextTarget::ConversationParticipantRef => self.push_conversation_participant_ref(text),
            TextTarget::ConversationMessageFlowRef => self.push_conversation_message_flow_ref(text),
            TextTarget::ChoreographyParticipantRef => {
                self.push_choreography_participant_ref(text);
            }
            TextTarget::ChoreographyMessageFlowRef => {
                self.push_choreography_message_flow_ref(text);
            }
            TextTarget::TextAnnotationText => self.append_text_annotation_text(text),
            TextTarget::CorrelationKeyPropertyRef => {
                self.push_conversation_correlation_property_ref(text);
            }
            TextTarget::ParticipantAssociationInnerRef => {
                self.set_participant_association_inner_ref(text);
            }
            TextTarget::ParticipantAssociationOuterRef => {
                self.set_participant_association_outer_ref(text);
            }
            TextTarget::CollaborationChoreographyRef => self.push_choreography_ref(text),
        }
    }

    pub(super) fn push_lane_flow_node_ref(&mut self, text: &str) {
        let Some((process_index, lane_set_index, lane_index)) = self.lane_stack.last().copied()
        else {
            return;
        };
        let Some(lane_set) = self.lane_set_mut(process_index, lane_set_index) else {
            return;
        };
        let Some(lane) = lane_set.lanes.get_mut(lane_index) else {
            return;
        };
        lane.flow_node_refs.push(text.to_string());
    }

    pub(super) fn push_data_association_source_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.source_refs.push(text.to_string());
    }

    pub(super) fn set_data_association_target_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.target_ref = Some(text.to_string());
    }

    pub(super) fn append_data_association_transformation(&mut self, text: &str) {
        let Some(association) = self.current_data_association_mut() else {
            return;
        };
        let Some(expression) = association.transformation.as_mut() else {
            return;
        };
        expression
            .body
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn append_data_association_assignment_from(&mut self, text: &str) {
        let Some(assignment) = self.current_data_association_assignment_mut() else {
            return;
        };
        let Some(expression) = assignment.from.as_mut() else {
            return;
        };
        expression
            .body
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn append_data_association_assignment_to(&mut self, text: &str) {
        let Some(assignment) = self.current_data_association_assignment_mut() else {
            return;
        };
        let Some(expression) = assignment.to.as_mut() else {
            return;
        };
        expression
            .body
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn append_correlation_message_path(&mut self, text: &str) {
        let Some((_, retrieval_expression)) =
            self.current_correlation_retrieval_expression.as_mut()
        else {
            return;
        };
        retrieval_expression
            .message_path
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn append_correlation_binding_data_path(&mut self, text: &str) {
        let Some(binding) = self.current_correlation_property_binding_mut() else {
            return;
        };
        binding
            .data_path
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn set_resource_role_resource_ref(&mut self, text: &str) {
        let Some((owner, role_index)) = self.current_resource_role else {
            return;
        };
        let Some(role) = self.resource_role_mut(owner, role_index) else {
            return;
        };
        role.resource_ref = Some(text.to_string());
    }

    pub(super) fn append_resource_role_assignment_expression(&mut self, text: &str) {
        let Some(role) = self.current_resource_assignment_expression_mut() else {
            return;
        };
        role.assignment_expression
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn append_resource_parameter_binding_expression(&mut self, text: &str) {
        let Some(binding) = self.current_resource_parameter_binding_mut() else {
            return;
        };
        binding
            .expression
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    pub(super) fn push_flow_element_category_value_ref(&mut self, text: &str) {
        if !self.collecting_flow_element_category_value_ref {
            return;
        }
        let Some((_, metadata)) = self.current_flow_element_metadata.as_mut() else {
            return;
        };
        metadata.category_value_refs.push(text.to_string());
    }

    pub(super) fn set_operation_in_message_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.in_message_ref = Some(text.to_string());
    }

    pub(super) fn set_operation_out_message_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.out_message_ref = Some(text.to_string());
    }

    pub(super) fn push_operation_error_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.error_refs.push(text.to_string());
    }

    pub(super) fn push_relationship_source_ref(&mut self, text: &str) {
        let Some(relationship) = self.current_relationship_mut() else {
            return;
        };
        relationship.source_refs.push(text.to_string());
    }

    pub(super) fn push_relationship_target_ref(&mut self, text: &str) {
        let Some(relationship) = self.current_relationship_mut() else {
            return;
        };
        relationship.target_refs.push(text.to_string());
    }

    pub(super) fn push_participant_interface_ref(&mut self, text: &str) {
        let Some(participant) = self.current_participant_mut() else {
            return;
        };
        participant.interface_refs.push(text.to_string());
    }

    pub(super) fn push_participant_end_point_ref(&mut self, text: &str) {
        let Some(participant) = self.current_participant_mut() else {
            return;
        };
        participant.end_point_refs.push(text.to_string());
    }

    pub(super) fn push_partner_entity_participant_ref(&mut self, text: &str) {
        let Some(partner_entity_index) = self.current_partner_entity else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(partner_entity) = root.partner_entities.get_mut(partner_entity_index) else {
            return;
        };
        partner_entity.participant_refs.push(text.to_string());
    }

    pub(super) fn push_partner_role_participant_ref(&mut self, text: &str) {
        let Some(partner_role_index) = self.current_partner_role else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(partner_role) = root.partner_roles.get_mut(partner_role_index) else {
            return;
        };
        partner_role.participant_refs.push(text.to_string());
    }

    pub(super) fn push_global_task_supported_interface_ref(&mut self, text: &str) {
        let Some(task) = self.current_global_task_mut() else {
            return;
        };
        task.supported_interface_refs.push(text.to_string());
    }

    pub(super) fn append_global_task_script(&mut self, text: &str) {
        let Some(task) = self.current_global_task_mut() else {
            return;
        };
        task.script.get_or_insert_with(String::new).push_str(text);
    }

    pub(super) fn push_process_support_ref(&mut self, text: &str) {
        let Some(process) = self.current_process_mut() else {
            return;
        };
        process.support_count += 1;
        process.supports.push(text.to_string());
    }

    pub(super) fn push_conversation_participant_ref(&mut self, text: &str) {
        let Some(conversation) = self.current_conversation_node_mut() else {
            return;
        };
        conversation.participant_refs.push(text.to_string());
    }

    pub(super) fn push_conversation_message_flow_ref(&mut self, text: &str) {
        let Some(conversation) = self.current_conversation_node_mut() else {
            return;
        };
        conversation.message_flow_refs.push(text.to_string());
    }

    pub(super) fn push_choreography_participant_ref(&mut self, text: &str) {
        let Some(activity) = self.current_choreography_activity_mut() else {
            return;
        };
        activity.participant_refs.push(text.to_string());
    }

    pub(super) fn push_choreography_message_flow_ref(&mut self, text: &str) {
        let Some(activity) = self.current_choreography_activity_mut() else {
            return;
        };
        activity.message_flow_refs.push(text.to_string());
    }

    pub(super) fn push_conversation_correlation_property_ref(&mut self, text: &str) {
        let Some((_, key)) = self.current_conversation_correlation_key.as_mut() else {
            return;
        };
        key.correlation_property_refs.push(text.to_string());
    }

    pub(super) fn set_participant_association_inner_ref(&mut self, text: &str) {
        let Some((_, association)) = self.current_participant_association.as_mut() else {
            return;
        };
        association.inner_participant_ref = Some(text.to_string());
    }

    pub(super) fn set_participant_association_outer_ref(&mut self, text: &str) {
        let Some((_, association)) = self.current_participant_association.as_mut() else {
            return;
        };
        association.outer_participant_ref = Some(text.to_string());
    }

    pub(super) fn push_choreography_ref(&mut self, text: &str) {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return;
        };
        collaboration.choreography_refs.push(text.to_string());
    }

    pub(super) fn append_extension_documentation(&mut self, text: &str) {
        let Some((_, documentation)) = self.current_extension_documentation.as_mut() else {
            return;
        };
        if !documentation.is_empty() {
            documentation.push(' ');
        }
        documentation.push_str(text);
    }

    pub(super) fn append_text_annotation_text(&mut self, text: &str) {
        let Some((_, annotation)) = self.current_text_annotation.as_mut() else {
            return;
        };
        let payload = annotation.text.get_or_insert_with(String::new);
        if !payload.is_empty() {
            payload.push(' ');
        }
        payload.push_str(text);
    }
}
