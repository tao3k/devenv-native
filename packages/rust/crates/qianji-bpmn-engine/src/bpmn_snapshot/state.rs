use super::xml::{attribute_value, boolean_attribute_value, bpmn_model_namespace, local_name};
use crate::bpmn_model_api::{
    BpmnCollaborationSnapshot, BpmnCorrelationPropertySnapshot,
    BpmnCorrelationRetrievalExpressionSnapshot, BpmnDataAssociationSnapshot,
    BpmnDataInputOutputSnapshot, BpmnDataObjectReferenceSnapshot, BpmnDataObjectSnapshot,
    BpmnDataStoreReferenceSnapshot, BpmnDataStoreSnapshot, BpmnDocumentSnapshot,
    BpmnIoSpecificationSnapshot, BpmnLaneSetSnapshot, BpmnLaneSnapshot, BpmnMessageFlowSnapshot,
    BpmnMessageSnapshot, BpmnParticipantSnapshot, BpmnProcessSnapshot, BpmnRootSnapshot,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug, Clone, Copy)]
pub(super) enum TextTarget {
    LaneFlowNode,
    DataAssociationSource,
    DataAssociationTarget,
    CorrelationMessagePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataAssociationKind {
    Input,
    Output,
}

#[derive(Debug, Default)]
pub(super) struct BpmnSnapshotScanState {
    root: Option<BpmnRootSnapshot>,
    collaborations: Vec<BpmnCollaborationSnapshot>,
    processes: Vec<BpmnProcessSnapshot>,
    current_collaboration: Option<usize>,
    current_process: Option<usize>,
    lane_set_stack: Vec<(usize, usize)>,
    lane_stack: Vec<(usize, usize, usize)>,
    current_correlation_property: Option<usize>,
    current_correlation_retrieval_expression:
        Option<(usize, BpmnCorrelationRetrievalExpressionSnapshot)>,
    io_specification_stack: Vec<(usize, usize)>,
    current_data_association: Option<(usize, DataAssociationKind, BpmnDataAssociationSnapshot)>,
}

impl BpmnSnapshotScanState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn handle_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(root_from_event(source, reader, event)?);
        }

        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match tag {
            "collaboration" if parent_tag == Some("definitions") => {
                self.start_collaboration(source, reader, event, is_empty)
            }
            "participant" if parent_tag == Some("collaboration") => {
                self.capture_participant(source, reader, event)
            }
            "messageFlow" if parent_tag == Some("collaboration") => {
                self.capture_message_flow(source, reader, event)
            }
            "process" if parent_tag == Some("definitions") => {
                self.start_process(source, reader, event, is_empty)
            }
            "message" if parent_tag == Some("definitions") => {
                self.capture_message(source, reader, event)
            }
            "correlationProperty" if parent_tag == Some("definitions") => {
                self.capture_correlation_property(source, reader, event, is_empty)
            }
            "correlationPropertyRetrievalExpression"
                if self.current_correlation_property.is_some() =>
            {
                self.start_correlation_retrieval_expression(source, reader, event, is_empty)
            }
            "dataStore" if parent_tag == Some("definitions") => {
                self.capture_data_store(source, reader, event)
            }
            "laneSet" if self.current_process.is_some() => {
                self.start_lane_set(source, reader, event, is_empty)
            }
            "lane" if self.current_lane_set().is_some() => {
                self.start_lane(source, reader, event, is_empty)
            }
            "dataObject" if self.current_process.is_some() => {
                self.capture_data_object(source, reader, event)
            }
            "dataObjectReference" if self.current_process.is_some() => {
                self.capture_data_object_reference(source, reader, event)
            }
            "dataStoreReference" if self.current_process.is_some() => {
                self.capture_data_store_reference(source, reader, event)
            }
            "ioSpecification" if self.current_process.is_some() => {
                self.start_io_specification(source, reader, event, is_empty)
            }
            "dataInput" if self.current_io_specification().is_some() => {
                self.capture_io_data_input(source, reader, event)
            }
            "dataOutput" if self.current_io_specification().is_some() => {
                self.capture_io_data_output(source, reader, event)
            }
            "dataInputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Input,
                    is_empty,
                ),
            "dataOutputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Output,
                    is_empty,
                ),
            _ => Ok(()),
        }
    }

    pub(super) fn finish_end_event(&mut self, tag: &str) {
        match tag {
            "collaboration" => self.current_collaboration = None,
            "correlationProperty" => {
                self.finish_correlation_retrieval_expression();
                self.current_correlation_property = None;
            }
            "correlationPropertyRetrievalExpression" => {
                self.finish_correlation_retrieval_expression();
            }
            "process" => {
                self.current_process = None;
                self.lane_set_stack.clear();
                self.lane_stack.clear();
                self.io_specification_stack.clear();
            }
            "laneSet" => {
                let _ = self.lane_set_stack.pop();
            }
            "lane" => {
                let _ = self.lane_stack.pop();
            }
            "ioSpecification" => {
                let _ = self.io_specification_stack.pop();
            }
            "dataInputAssociation" => self.finish_data_association(DataAssociationKind::Input),
            "dataOutputAssociation" => self.finish_data_association(DataAssociationKind::Output),
            _ => {}
        }
    }

    pub(super) fn handle_text_chunk(&mut self, text: &str, target: Option<TextTarget>) {
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
            TextTarget::CorrelationMessagePath => self.append_correlation_message_path(text),
        }
    }

    pub(super) fn finish_pending(&mut self) {
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
    }

    pub(super) fn into_snapshot(self, source: &BpmnSourceFile) -> BpmnDocumentSnapshot {
        BpmnDocumentSnapshot {
            source_id: source.source_id.clone(),
            root: self
                .root
                .unwrap_or_else(|| crate::bpmn_model_api::empty_bpmn_root_snapshot(source)),
            collaborations: self.collaborations,
            processes: self.processes,
        }
    }

    fn start_collaboration(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let collaboration = BpmnCollaborationSnapshot {
            collaboration_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            participants: Vec::new(),
            message_flows: Vec::new(),
        };
        self.collaborations.push(collaboration);
        if let Some(root) = self.root.as_mut() {
            root.collaboration_count += 1;
        }
        if !is_empty {
            self.current_collaboration = self.collaborations.len().checked_sub(1);
        }
        Ok(())
    }

    fn capture_participant(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration.participants.push(BpmnParticipantSnapshot {
            participant_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            process_ref: attribute_value(source, reader, event, "processRef")?,
        });
        Ok(())
    }

    fn capture_message_flow(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration.message_flows.push(BpmnMessageFlowSnapshot {
            message_flow_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            source_ref: attribute_value(source, reader, event, "sourceRef")?,
            target_ref: attribute_value(source, reader, event, "targetRef")?,
            message_ref: attribute_value(source, reader, event, "messageRef")?,
        });
        Ok(())
    }

    fn capture_message(
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

    fn capture_correlation_property(
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

    fn start_correlation_retrieval_expression(
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

    fn start_process(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let process = BpmnProcessSnapshot {
            process_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            is_executable: boolean_attribute_value(source, reader, event, "isExecutable")?,
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
            data_input_association_count: 0,
            data_input_associations: Vec::new(),
            data_output_association_count: 0,
            data_output_associations: Vec::new(),
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

    fn capture_data_store(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.data_store_count += 1;
        root.data_stores.push(BpmnDataStoreSnapshot {
            data_store_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            capacity: attribute_value(source, reader, event, "capacity")?,
            is_unlimited: boolean_attribute_value(source, reader, event, "isUnlimited")?,
        });
        Ok(())
    }

    fn start_lane_set(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let lane_set = BpmnLaneSetSnapshot {
            lane_set_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            lanes: Vec::new(),
        };
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.lane_set_count += 1;
        process.lane_sets.push(lane_set);
        if !is_empty {
            let lane_set_index = process.lane_sets.len().saturating_sub(1);
            self.lane_set_stack.push((process_index, lane_set_index));
        }
        Ok(())
    }

    fn start_lane(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((process_index, lane_set_index)) = self.current_lane_set() else {
            return Ok(());
        };
        let lane = BpmnLaneSnapshot {
            lane_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            flow_node_refs: Vec::new(),
        };
        let Some(lane_set) = self.lane_set_mut(process_index, lane_set_index) else {
            return Ok(());
        };
        lane_set.lanes.push(lane);
        if !is_empty {
            let lane_index = lane_set.lanes.len().saturating_sub(1);
            self.lane_stack
                .push((process_index, lane_set_index, lane_index));
        }
        Ok(())
    }

    fn capture_data_object(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_count += 1;
        process.data_objects.push(BpmnDataObjectSnapshot {
            data_object_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
        });
        Ok(())
    }

    fn capture_data_object_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_reference_count += 1;
        process
            .data_object_references
            .push(BpmnDataObjectReferenceSnapshot {
                data_object_reference_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                data_object_ref: attribute_value(source, reader, event, "dataObjectRef")?,
            });
        Ok(())
    }

    fn capture_data_store_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_store_reference_count += 1;
        process
            .data_store_references
            .push(BpmnDataStoreReferenceSnapshot {
                data_store_reference_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                data_store_ref: attribute_value(source, reader, event, "dataStoreRef")?,
            });
        Ok(())
    }

    fn start_io_specification(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.io_specification_count += 1;
        process.io_specifications.push(BpmnIoSpecificationSnapshot {
            io_specification_id: attribute_value(source, reader, event, "id")?,
            data_inputs: Vec::new(),
            data_outputs: Vec::new(),
        });
        if !is_empty {
            let io_index = process.io_specifications.len().saturating_sub(1);
            self.io_specification_stack.push((process_index, io_index));
        }
        Ok(())
    }

    fn capture_io_data_input(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_inputs
            .push(data_input_output_from_event(source, reader, event)?);
        Ok(())
    }

    fn capture_io_data_output(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_outputs
            .push(data_input_output_from_event(source, reader, event)?);
        Ok(())
    }

    fn start_data_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        kind: DataAssociationKind,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let association = BpmnDataAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            source_refs: Vec::new(),
            target_ref: None,
        };
        if is_empty {
            self.push_data_association(process_index, kind, association);
            return Ok(());
        }
        self.current_data_association = Some((process_index, kind, association));
        Ok(())
    }

    fn push_lane_flow_node_ref(&mut self, text: &str) {
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

    fn push_data_association_source_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.source_refs.push(text.to_string());
    }

    fn set_data_association_target_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.target_ref = Some(text.to_string());
    }

    fn append_correlation_message_path(&mut self, text: &str) {
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

    fn finish_correlation_retrieval_expression(&mut self) {
        let Some((property_index, retrieval_expression)) =
            self.current_correlation_retrieval_expression.take()
        else {
            return;
        };
        self.push_correlation_retrieval_expression(property_index, retrieval_expression);
    }

    fn push_correlation_retrieval_expression(
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

    fn finish_data_association(&mut self, expected_kind: DataAssociationKind) {
        let Some((process_index, kind, association)) = self.current_data_association.take() else {
            return;
        };
        if kind != expected_kind {
            self.current_data_association = Some((process_index, kind, association));
            return;
        }
        self.push_data_association(process_index, kind, association);
    }

    fn push_data_association(
        &mut self,
        process_index: usize,
        kind: DataAssociationKind,
        association: BpmnDataAssociationSnapshot,
    ) {
        let Some(process) = self.processes.get_mut(process_index) else {
            return;
        };
        match kind {
            DataAssociationKind::Input => {
                process.data_input_association_count += 1;
                process.data_input_associations.push(association);
            }
            DataAssociationKind::Output => {
                process.data_output_association_count += 1;
                process.data_output_associations.push(association);
            }
        }
    }

    fn current_collaboration_mut(&mut self) -> Option<&mut BpmnCollaborationSnapshot> {
        self.current_collaboration
            .and_then(|index| self.collaborations.get_mut(index))
    }

    fn current_process_mut(&mut self) -> Option<&mut BpmnProcessSnapshot> {
        self.current_process
            .and_then(|index| self.processes.get_mut(index))
    }

    fn current_lane_set(&self) -> Option<(usize, usize)> {
        self.lane_set_stack.last().copied()
    }

    fn lane_set_mut(
        &mut self,
        process_index: usize,
        lane_set_index: usize,
    ) -> Option<&mut BpmnLaneSetSnapshot> {
        self.processes
            .get_mut(process_index)?
            .lane_sets
            .get_mut(lane_set_index)
    }

    fn current_io_specification(&self) -> Option<(usize, usize)> {
        self.io_specification_stack.last().copied()
    }

    fn current_io_specification_mut(&mut self) -> Option<&mut BpmnIoSpecificationSnapshot> {
        let (process_index, io_index) = self.current_io_specification()?;
        self.processes
            .get_mut(process_index)?
            .io_specifications
            .get_mut(io_index)
    }
}

fn root_from_event(
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
        collaboration_count: 0,
        process_count: 0,
        message_count: 0,
        messages: Vec::new(),
        correlation_property_count: 0,
        correlation_properties: Vec::new(),
        data_store_count: 0,
        data_stores: Vec::new(),
    })
}

fn data_input_output_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnDataInputOutputSnapshot> {
    Ok(BpmnDataInputOutputSnapshot {
        data_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
        is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
    })
}
