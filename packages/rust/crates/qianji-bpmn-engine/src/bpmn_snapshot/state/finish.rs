use super::{
    BpmnDocumentSnapshot, BpmnSnapshotScanState, BpmnSourceFile, DataAssociationKind,
    is_flow_element_metadata_owner_tag, is_global_task_tag, is_resource_role_tag,
};

impl BpmnSnapshotScanState {
    pub(in crate::bpmn_snapshot) fn finish_end_event(&mut self, tag: &str) {
        match tag {
            "collaboration" | "globalConversation" | "choreography" | "globalChoreographyTask" => {
                self.finish_collaboration_root();
            }
            "participant" => self.current_participant = None,
            "partnerEntity" => self.current_partner_entity = None,
            "partnerRole" => self.current_partner_role = None,
            tag if is_global_task_tag(tag) => self.current_global_task = None,
            tag if is_resource_role_tag(tag) => {
                self.current_resource_assignment_expression = None;
                self.current_resource_parameter_binding = None;
                self.current_resource_role = None;
            }
            tag if is_flow_element_metadata_owner_tag(tag) => {
                self.finish_flow_element_metadata();
            }
            "categoryValueRef" => self.collecting_flow_element_category_value_ref = false,
            "conversation" | "subConversation" | "callConversation" => {
                self.finish_conversation_node();
            }
            "choreographyTask" | "subChoreography" | "callChoreography" => {
                self.finish_choreography_activity();
            }
            "textAnnotation" => self.finish_text_annotation(),
            "correlationKey" => self.finish_conversation_correlation_key(),
            "participantAssociation" => self.finish_participant_association(),
            "correlationProperty" => {
                self.finish_correlation_retrieval_expression();
                self.current_correlation_property = None;
            }
            "correlationPropertyRetrievalExpression" => {
                self.finish_correlation_retrieval_expression();
            }
            "operation" => self.current_operation = None,
            "interface" => self.finish_interface(),
            "documentation" => self.finish_extension_documentation(),
            "extension" => self.finish_extension(),
            "BPMNLabel" => self.current_label = None,
            "BPMNLabelStyle" => self.current_label_style = None,
            "BPMNShape" => self.current_shape = None,
            "BPMNEdge" => self.current_edge = None,
            "BPMNPlane" => self.finish_bpmn_plane(),
            "BPMNDiagram" => self.finish_bpmn_diagram(),
            "resource" => self.current_resource = None,
            "category" => self.current_category = None,
            "relationship" => self.current_relationship = None,
            "process" => self.finish_process(),
            "correlationPropertyBinding" => self.current_correlation_property_binding = None,
            "correlationSubscription" => self.finish_correlation_subscription(),
            "resourceAssignmentExpression" => self.current_resource_assignment_expression = None,
            "resourceParameterBinding" => self.current_resource_parameter_binding = None,
            "laneSet" => {
                let _ = self.lane_set_stack.pop();
            }
            "lane" => {
                let _ = self.lane_stack.pop();
            }
            "ioSpecification" => {
                self.current_io_set = None;
                let _ = self.io_specification_stack.pop();
            }
            "inputSet" | "outputSet" => self.current_io_set = None,
            "dataInput"
            | "dataOutput"
            | "dataObject"
            | "dataObjectReference"
            | "dataStore"
            | "dataStoreReference" => self.current_data_state_owner = None,
            "assignment" => self.current_data_association_assignment = None,
            "dataInputAssociation" => self.finish_data_association(DataAssociationKind::Input),
            "dataOutputAssociation" => self.finish_data_association(DataAssociationKind::Output),
            _ => {}
        }
    }

    pub(super) fn finish_collaboration_root(&mut self) {
        self.finish_conversation_correlation_key();
        self.finish_participant_association();
        self.current_participant = None;
        self.conversation_node_stack.clear();
        self.choreography_activity_stack.clear();
        self.current_collaboration = None;
    }

    pub(super) fn finish_conversation_node(&mut self) {
        self.finish_conversation_correlation_key();
        self.finish_participant_association();
        let _ = self.conversation_node_stack.pop();
    }

    pub(super) fn finish_choreography_activity(&mut self) {
        self.finish_conversation_correlation_key();
        self.finish_participant_association();
        let _ = self.choreography_activity_stack.pop();
    }

    pub(super) fn finish_interface(&mut self) {
        self.current_operation = None;
        self.current_interface = None;
    }

    pub(super) fn finish_extension(&mut self) {
        self.finish_extension_documentation();
        self.current_extension = None;
    }

    pub(super) fn finish_bpmn_plane(&mut self) {
        self.current_label = None;
        self.current_shape = None;
        self.current_edge = None;
        self.current_plane = None;
    }

    pub(super) fn finish_bpmn_diagram(&mut self) {
        self.current_label = None;
        self.current_label_style = None;
        self.current_shape = None;
        self.current_edge = None;
        self.current_plane = None;
        self.current_diagram = None;
    }

    pub(super) fn finish_process(&mut self) {
        self.finish_flow_element_metadata();
        self.current_process = None;
        self.current_correlation_property_binding = None;
        self.current_correlation_subscription = None;
        self.lane_set_stack.clear();
        self.lane_stack.clear();
        self.io_specification_stack.clear();
        self.current_io_set = None;
        self.current_data_association = None;
        self.current_data_association_assignment = None;
    }

    pub(super) fn finish_correlation_subscription(&mut self) {
        self.current_correlation_property_binding = None;
        self.current_correlation_subscription = None;
    }

    pub(in crate::bpmn_snapshot) fn finish_pending(&mut self) {
        if self
            .current_data_association
            .as_ref()
            .is_some_and(|(_, kind, _)| *kind == DataAssociationKind::Input)
        {
            self.finish_data_association(DataAssociationKind::Input);
        }
        if self.current_data_association.is_some() {
            self.finish_data_association(DataAssociationKind::Output);
        }
        self.finish_correlation_retrieval_expression();
        self.current_operation = None;
        self.current_interface = None;
        self.current_resource = None;
        self.current_category = None;
        self.finish_extension_documentation();
        self.current_extension = None;
        self.finish_text_annotation();
        self.finish_conversation_correlation_key();
        self.finish_participant_association();
        self.current_correlation_property_binding = None;
        self.current_correlation_subscription = None;
        self.current_resource_assignment_expression = None;
        self.current_resource_parameter_binding = None;
        self.current_resource_role = None;
        self.current_data_state_owner = None;
        self.finish_flow_element_metadata();
        self.collecting_flow_element_category_value_ref = false;
        self.current_partner_entity = None;
        self.current_partner_role = None;
        self.current_global_task = None;
        self.io_specification_stack.clear();
        self.current_participant = None;
        self.conversation_node_stack.clear();
        self.choreography_activity_stack.clear();
        self.current_relationship = None;
        self.current_label = None;
        self.current_label_style = None;
        self.current_shape = None;
        self.current_edge = None;
        self.current_plane = None;
        self.current_diagram = None;
    }

    pub(in crate::bpmn_snapshot) fn into_snapshot(
        self,
        source: &BpmnSourceFile,
    ) -> BpmnDocumentSnapshot {
        BpmnDocumentSnapshot {
            source_id: source.source_id.clone(),
            root: self
                .root
                .unwrap_or_else(|| crate::bpmn_model_api::empty_bpmn_root_snapshot(source)),
            collaborations: self.collaborations,
            processes: self.processes,
        }
    }
}
