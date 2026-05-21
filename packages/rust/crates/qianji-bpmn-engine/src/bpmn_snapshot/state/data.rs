use super::{
    BpmnDataAssociationAssignmentSnapshot, BpmnDataAssociationSnapshot,
    BpmnDataInputOutputSnapshot, BpmnDataObjectReferenceSnapshot, BpmnDataObjectSnapshot,
    BpmnDataStoreReferenceSnapshot, BpmnDataStoreSnapshot, BpmnInputSetSnapshot,
    BpmnIoSpecificationSnapshot, BpmnLaneSetSnapshot, BpmnLaneSnapshot, BpmnOutputSetSnapshot,
    BpmnSnapshotScanState, BpmnSourceFile, BytesStart, DataAssociationAssignmentExpressionKind,
    DataAssociationKind, DataStateOwner, IoSetKind, IoSpecificationOwner, Reader, Result,
    attribute_value, boolean_attribute_value, data_association_expression_from_event,
    data_input_output_from_event, data_state_from_event, io_binding_from_event,
};

impl BpmnSnapshotScanState {
    pub(super) fn start_data_store(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.data_store_count += 1;
        root.data_stores.push(BpmnDataStoreSnapshot {
            data_store_id: attribute_value(source, reader, event, "id")?.map(Into::into),
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            capacity: attribute_value(source, reader, event, "capacity")?,
            is_unlimited: boolean_attribute_value(source, reader, event, "isUnlimited")?
                .map(Into::into),
            data_state: None,
        });
        if !is_empty {
            let data_store_index = root.data_stores.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::RootDataStore(data_store_index));
        }
        Ok(())
    }

    pub(super) fn start_lane_set(
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

    pub(super) fn start_lane(
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

    pub(super) fn start_data_object(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_count += 1;
        process.data_objects.push(BpmnDataObjectSnapshot {
            data_object_id: attribute_value(source, reader, event, "id")?.map(Into::into),
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            is_collection: boolean_attribute_value(source, reader, event, "isCollection")?
                .map(Into::into),
            data_state: None,
        });
        if !is_empty {
            let data_object_index = process.data_objects.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::ProcessDataObject(
                process_index,
                data_object_index,
            ));
        }
        Ok(())
    }

    pub(super) fn start_data_object_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_reference_count += 1;
        process
            .data_object_references
            .push(BpmnDataObjectReferenceSnapshot {
                data_object_reference_id: attribute_value(source, reader, event, "id")?
                    .map(Into::into),
                name: attribute_value(source, reader, event, "name")?,
                data_object_ref: attribute_value(source, reader, event, "dataObjectRef")?,
                item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
                data_state: None,
            });
        if !is_empty {
            let reference_index = process.data_object_references.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::ProcessDataObjectReference(
                process_index,
                reference_index,
            ));
        }
        Ok(())
    }

    pub(super) fn start_data_store_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_store_reference_count += 1;
        process
            .data_store_references
            .push(BpmnDataStoreReferenceSnapshot {
                data_store_reference_id: attribute_value(source, reader, event, "id")?
                    .map(Into::into),
                name: attribute_value(source, reader, event, "name")?,
                data_store_ref: attribute_value(source, reader, event, "dataStoreRef")?,
                item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
                data_state: None,
            });
        if !is_empty {
            let reference_index = process.data_store_references.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::ProcessDataStoreReference(
                process_index,
                reference_index,
            ));
        }
        Ok(())
    }

    pub(super) fn attach_data_state(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let state = data_state_from_event(source, reader, event)?;
        let Some(owner) = self.current_data_state_owner else {
            return Ok(());
        };
        match owner {
            DataStateOwner::RootDataStore(data_store_index) => {
                if let Some(data_store) = self
                    .root
                    .as_mut()
                    .and_then(|root| root.data_stores.get_mut(data_store_index))
                {
                    data_store.data_state = Some(state);
                }
            }
            DataStateOwner::ProcessDataObject(process_index, data_object_index) => {
                if let Some(data_object) = self
                    .processes
                    .get_mut(process_index)
                    .and_then(|process| process.data_objects.get_mut(data_object_index))
                {
                    data_object.data_state = Some(state);
                }
            }
            DataStateOwner::ProcessDataObjectReference(process_index, reference_index) => {
                if let Some(reference) = self
                    .processes
                    .get_mut(process_index)
                    .and_then(|process| process.data_object_references.get_mut(reference_index))
                {
                    reference.data_state = Some(state);
                }
            }
            DataStateOwner::ProcessDataStoreReference(process_index, reference_index) => {
                if let Some(reference) = self
                    .processes
                    .get_mut(process_index)
                    .and_then(|process| process.data_store_references.get_mut(reference_index))
                {
                    reference.data_state = Some(state);
                }
            }
            DataStateOwner::IoDataInput(io_owner, data_index) => {
                if let Some(input) = self.io_data_input_mut(io_owner, data_index) {
                    input.data_state = Some(state);
                }
            }
            DataStateOwner::IoDataOutput(io_owner, data_index) => {
                if let Some(output) = self.io_data_output_mut(io_owner, data_index) {
                    output.data_state = Some(state);
                }
            }
        }
        Ok(())
    }

    pub(super) fn start_io_specification(
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
            input_sets: Vec::new(),
            output_sets: Vec::new(),
        });
        if !is_empty {
            let io_index = process.io_specifications.len().saturating_sub(1);
            self.io_specification_stack
                .push(IoSpecificationOwner::Process(process_index, io_index));
        }
        Ok(())
    }

    pub(super) fn start_global_task_io_specification(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(task_index) = self.current_global_task else {
            return Ok(());
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(task) = root.global_tasks.get_mut(task_index) else {
            return Ok(());
        };
        task.io_specification_count += 1;
        task.io_specifications.push(BpmnIoSpecificationSnapshot {
            io_specification_id: attribute_value(source, reader, event, "id")?,
            data_inputs: Vec::new(),
            data_outputs: Vec::new(),
            input_sets: Vec::new(),
            output_sets: Vec::new(),
        });
        if !is_empty {
            let io_index = task.io_specifications.len().saturating_sub(1);
            self.io_specification_stack
                .push(IoSpecificationOwner::GlobalTask(task_index, io_index));
        }
        Ok(())
    }

    pub(super) fn capture_process_io_binding(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let binding = io_binding_from_event(source, reader, event)?;
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.io_binding_count += 1;
        process.io_bindings.push(binding);
        Ok(())
    }

    pub(super) fn capture_global_task_io_binding(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let binding = io_binding_from_event(source, reader, event)?;
        let Some(task) = self.current_global_task_mut() else {
            return Ok(());
        };
        task.io_binding_count += 1;
        task.io_bindings.push(binding);
        Ok(())
    }

    pub(super) fn capture_io_data_input(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_io_specification() else {
            return Ok(());
        };
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_inputs
            .push(data_input_output_from_event(source, reader, event)?);
        if !is_empty {
            let data_index = io_specification.data_inputs.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::IoDataInput(owner, data_index));
        }
        Ok(())
    }

    pub(super) fn capture_io_data_output(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_io_specification() else {
            return Ok(());
        };
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_outputs
            .push(data_input_output_from_event(source, reader, event)?);
        if !is_empty {
            let data_index = io_specification.data_outputs.len().saturating_sub(1);
            self.current_data_state_owner = Some(DataStateOwner::IoDataOutput(owner, data_index));
        }
        Ok(())
    }

    pub(super) fn start_io_input_set(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.current_io_set = None;
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification.input_sets.push(BpmnInputSetSnapshot {
            set_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            data_input_refs: Vec::new(),
            optional_input_refs: Vec::new(),
            while_executing_input_refs: Vec::new(),
            output_set_refs: Vec::new(),
        });
        if !is_empty {
            self.current_io_set = Some((
                IoSetKind::Input,
                io_specification.input_sets.len().saturating_sub(1),
            ));
        }
        Ok(())
    }

    pub(super) fn start_io_output_set(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.current_io_set = None;
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification.output_sets.push(BpmnOutputSetSnapshot {
            set_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            data_output_refs: Vec::new(),
            optional_output_refs: Vec::new(),
            while_executing_output_refs: Vec::new(),
            input_set_refs: Vec::new(),
        });
        if !is_empty {
            self.current_io_set = Some((
                IoSetKind::Output,
                io_specification.output_sets.len().saturating_sub(1),
            ));
        }
        Ok(())
    }

    pub(super) fn push_io_input_set_data_input_ref(&mut self, text: &str) {
        let Some(input_set) = self.current_io_input_set_mut() else {
            return;
        };
        input_set.data_input_refs.push(text.to_string());
    }

    pub(super) fn push_io_input_set_optional_input_ref(&mut self, text: &str) {
        let Some(input_set) = self.current_io_input_set_mut() else {
            return;
        };
        input_set.optional_input_refs.push(text.to_string());
    }

    pub(super) fn push_io_input_set_while_executing_input_ref(&mut self, text: &str) {
        let Some(input_set) = self.current_io_input_set_mut() else {
            return;
        };
        input_set.while_executing_input_refs.push(text.to_string());
    }

    pub(super) fn push_io_input_set_output_set_ref(&mut self, text: &str) {
        let Some(input_set) = self.current_io_input_set_mut() else {
            return;
        };
        input_set.output_set_refs.push(text.to_string());
    }

    pub(super) fn push_io_output_set_data_output_ref(&mut self, text: &str) {
        let Some(output_set) = self.current_io_output_set_mut() else {
            return;
        };
        output_set.data_output_refs.push(text.to_string());
    }

    pub(super) fn push_io_output_set_optional_output_ref(&mut self, text: &str) {
        let Some(output_set) = self.current_io_output_set_mut() else {
            return;
        };
        output_set.optional_output_refs.push(text.to_string());
    }

    pub(super) fn push_io_output_set_while_executing_output_ref(&mut self, text: &str) {
        let Some(output_set) = self.current_io_output_set_mut() else {
            return;
        };
        output_set
            .while_executing_output_refs
            .push(text.to_string());
    }

    pub(super) fn push_io_output_set_input_set_ref(&mut self, text: &str) {
        let Some(output_set) = self.current_io_output_set_mut() else {
            return;
        };
        output_set.input_set_refs.push(text.to_string());
    }

    pub(super) fn start_data_association(
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
            transformation: None,
            assignments: Vec::new(),
        };
        if is_empty {
            self.push_data_association(process_index, kind, association);
            return Ok(());
        }
        self.current_data_association = Some((process_index, kind, association));
        Ok(())
    }

    pub(super) fn start_data_association_transformation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let expression = data_association_expression_from_event(source, reader, event)?;
        let Some(association) = self.current_data_association_mut() else {
            return Ok(());
        };
        association.transformation = Some(expression);
        Ok(())
    }

    pub(super) fn start_data_association_assignment(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.current_data_association_assignment = None;
        let assignment = BpmnDataAssociationAssignmentSnapshot {
            assignment_id: attribute_value(source, reader, event, "id")?,
            from: None,
            to: None,
        };
        let Some(association) = self.current_data_association_mut() else {
            return Ok(());
        };
        association.assignments.push(assignment);
        if !is_empty {
            self.current_data_association_assignment =
                Some(association.assignments.len().saturating_sub(1));
        }
        Ok(())
    }

    pub(super) fn start_data_association_assignment_expression(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        kind: DataAssociationAssignmentExpressionKind,
    ) -> Result<()> {
        let expression = data_association_expression_from_event(source, reader, event)?;
        let Some(assignment) = self.current_data_association_assignment_mut() else {
            return Ok(());
        };
        match kind {
            DataAssociationAssignmentExpressionKind::From => assignment.from = Some(expression),
            DataAssociationAssignmentExpressionKind::To => assignment.to = Some(expression),
        }
        Ok(())
    }

    pub(super) fn current_data_association_mut(
        &mut self,
    ) -> Option<&mut BpmnDataAssociationSnapshot> {
        self.current_data_association
            .as_mut()
            .map(|(_, _, association)| association)
    }

    pub(super) fn current_data_association_assignment_mut(
        &mut self,
    ) -> Option<&mut BpmnDataAssociationAssignmentSnapshot> {
        let assignment_index = self.current_data_association_assignment?;
        let association = self.current_data_association_mut()?;
        association.assignments.get_mut(assignment_index)
    }

    pub(super) fn finish_data_association(&mut self, expected_kind: DataAssociationKind) {
        let Some((process_index, kind, association)) = self.current_data_association.take() else {
            return;
        };
        if kind != expected_kind {
            self.current_data_association = Some((process_index, kind, association));
            return;
        }
        self.current_data_association_assignment = None;
        self.push_data_association(process_index, kind, association);
    }

    pub(super) fn push_data_association(
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

    pub(super) fn current_lane_set(&self) -> Option<(usize, usize)> {
        self.lane_set_stack.last().copied()
    }

    pub(super) fn lane_set_mut(
        &mut self,
        process_index: usize,
        lane_set_index: usize,
    ) -> Option<&mut BpmnLaneSetSnapshot> {
        self.processes
            .get_mut(process_index)?
            .lane_sets
            .get_mut(lane_set_index)
    }

    pub(super) fn current_io_specification(&self) -> Option<IoSpecificationOwner> {
        self.io_specification_stack.last().copied()
    }

    pub(super) fn current_io_specification_mut(
        &mut self,
    ) -> Option<&mut BpmnIoSpecificationSnapshot> {
        match self.current_io_specification()? {
            IoSpecificationOwner::Process(process_index, io_index) => self
                .processes
                .get_mut(process_index)?
                .io_specifications
                .get_mut(io_index),
            IoSpecificationOwner::GlobalTask(task_index, io_index) => self
                .root
                .as_mut()?
                .global_tasks
                .get_mut(task_index)?
                .io_specifications
                .get_mut(io_index),
        }
    }

    pub(super) fn current_io_input_set_mut(&mut self) -> Option<&mut BpmnInputSetSnapshot> {
        let (kind, set_index) = self.current_io_set?;
        if kind != IoSetKind::Input {
            return None;
        }
        self.current_io_specification_mut()?
            .input_sets
            .get_mut(set_index)
    }

    pub(super) fn current_io_output_set_mut(&mut self) -> Option<&mut BpmnOutputSetSnapshot> {
        let (kind, set_index) = self.current_io_set?;
        if kind != IoSetKind::Output {
            return None;
        }
        self.current_io_specification_mut()?
            .output_sets
            .get_mut(set_index)
    }

    pub(super) fn io_data_input_mut(
        &mut self,
        owner: IoSpecificationOwner,
        data_index: usize,
    ) -> Option<&mut BpmnDataInputOutputSnapshot> {
        match owner {
            IoSpecificationOwner::Process(process_index, io_index) => self
                .processes
                .get_mut(process_index)?
                .io_specifications
                .get_mut(io_index)?
                .data_inputs
                .get_mut(data_index),
            IoSpecificationOwner::GlobalTask(task_index, io_index) => self
                .root
                .as_mut()?
                .global_tasks
                .get_mut(task_index)?
                .io_specifications
                .get_mut(io_index)?
                .data_inputs
                .get_mut(data_index),
        }
    }

    pub(super) fn io_data_output_mut(
        &mut self,
        owner: IoSpecificationOwner,
        data_index: usize,
    ) -> Option<&mut BpmnDataInputOutputSnapshot> {
        match owner {
            IoSpecificationOwner::Process(process_index, io_index) => self
                .processes
                .get_mut(process_index)?
                .io_specifications
                .get_mut(io_index)?
                .data_outputs
                .get_mut(data_index),
            IoSpecificationOwner::GlobalTask(task_index, io_index) => self
                .root
                .as_mut()?
                .global_tasks
                .get_mut(task_index)?
                .io_specifications
                .get_mut(io_index)?
                .data_outputs
                .get_mut(data_index),
        }
    }
}
