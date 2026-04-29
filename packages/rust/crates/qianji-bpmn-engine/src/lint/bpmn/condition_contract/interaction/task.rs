use super::{
    ActiveNativeInteractionTask, BytesStart, NativeAssociationCapture, NativeInputAssociation,
    NativeOutputAssociation, Reader, StaticInteractionChoiceOutput, append_to_option,
    attribute_value, choice_values_from_assignment, local_name,
};

impl ActiveNativeInteractionTask {
    pub(in crate::lint::bpmn::condition_contract) fn new(task_id: String) -> Self {
        Self {
            task_id,
            ..Self::default()
        }
    }

    pub(in crate::lint::bpmn::condition_contract) fn handle_start(
        &mut self,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) {
        let event_name = event.name();
        let name = local_name(event_name.as_ref());
        match name {
            "dataInput" => self.record_data_input(reader, event),
            "dataOutput" => self.record_data_output(reader, event),
            "dataInputAssociation" => {
                self.active_input_association = Some(NativeInputAssociation::default());
            }
            "dataOutputAssociation" => {
                self.active_output_association = Some(NativeOutputAssociation::default());
            }
            "sourceRef" if self.active_input_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::InputSourceRef);
            }
            "targetRef" if self.active_input_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::InputTargetRef);
            }
            "from" if self.active_input_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::InputAssignmentFrom);
            }
            "to" if self.active_input_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::InputAssignmentTo);
            }
            "sourceRef" if self.active_output_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::OutputSourceRef);
            }
            "targetRef" if self.active_output_association.is_some() => {
                self.text_capture = Some(NativeAssociationCapture::OutputTargetRef);
            }
            _ => {}
        }
    }

    pub(in crate::lint::bpmn::condition_contract) fn handle_empty(
        &mut self,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) {
        let event_name = event.name();
        let name = local_name(event_name.as_ref());
        match name {
            "dataInput" => self.record_data_input(reader, event),
            "dataOutput" => self.record_data_output(reader, event),
            _ => {}
        }
    }

    pub(in crate::lint::bpmn::condition_contract) fn handle_end(&mut self, name: &str) {
        match name {
            "sourceRef" | "targetRef" | "from" | "to" => {
                self.text_capture = None;
            }
            "dataInputAssociation" => {
                if let Some(association) = self.active_input_association.take() {
                    self.input_associations.push(association);
                }
                self.text_capture = None;
            }
            "dataOutputAssociation" => {
                if let Some(association) = self.active_output_association.take() {
                    self.output_associations.push(association);
                }
                self.text_capture = None;
            }
            _ => {}
        }
    }

    pub(in crate::lint::bpmn::condition_contract) fn append_text(&mut self, text: &str) {
        let Some(capture) = self.text_capture.as_ref() else {
            return;
        };
        match capture {
            NativeAssociationCapture::InputSourceRef => {
                if let Some(association) = self.active_input_association.as_mut() {
                    append_to_option(&mut association.source_ref, text);
                }
            }
            NativeAssociationCapture::InputTargetRef => {
                if let Some(association) = self.active_input_association.as_mut() {
                    append_to_option(&mut association.target_ref, text);
                }
            }
            NativeAssociationCapture::InputAssignmentFrom => {
                if let Some(association) = self.active_input_association.as_mut() {
                    append_to_option(&mut association.assignment_from, text);
                }
            }
            NativeAssociationCapture::InputAssignmentTo => {
                if let Some(association) = self.active_input_association.as_mut() {
                    append_to_option(&mut association.assignment_to, text);
                }
            }
            NativeAssociationCapture::OutputSourceRef => {
                if let Some(association) = self.active_output_association.as_mut() {
                    append_to_option(&mut association.source_ref, text);
                }
            }
            NativeAssociationCapture::OutputTargetRef => {
                if let Some(association) = self.active_output_association.as_mut() {
                    append_to_option(&mut association.target_ref, text);
                }
            }
        }
    }

    pub(in crate::lint::bpmn::condition_contract) fn finish_output(
        mut self,
    ) -> Option<StaticInteractionChoiceOutput> {
        self.handle_end("dataInputAssociation");
        self.handle_end("dataOutputAssociation");
        let output = self.answer_output_target()?;
        let choice_values = self.static_choice_values();
        if choice_values.is_empty() {
            return None;
        }
        Some(StaticInteractionChoiceOutput {
            task_id: self.task_id,
            output,
            choice_values,
        })
    }

    fn record_data_input(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(id) = attribute_value(reader, event, "id")
            && let Some(name) = attribute_value(reader, event, "name")
        {
            self.data_inputs.insert(id, name);
        }
    }

    fn record_data_output(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(id) = attribute_value(reader, event, "id")
            && let Some(name) = attribute_value(reader, event, "name")
        {
            self.data_outputs.insert(id, name);
        }
    }

    fn answer_output_target(&self) -> Option<String> {
        self.output_associations.iter().find_map(|association| {
            let source_ref = association.source_ref.as_deref()?.trim();
            let output_name = self.data_outputs.get(source_ref)?;
            if output_name == "answer" {
                association
                    .target_ref
                    .as_deref()
                    .map(str::trim)
                    .filter(|target| !target.is_empty())
                    .map(ToOwned::to_owned)
            } else {
                None
            }
        })
    }

    fn static_choice_values(&self) -> Vec<String> {
        self.input_associations
            .iter()
            .filter_map(|association| {
                let target = association
                    .target_ref
                    .as_deref()
                    .or(association.assignment_to.as_deref())?
                    .trim();
                let input_name = self.data_inputs.get(target)?;
                if input_name == "choices" {
                    association.assignment_from.as_deref()
                } else {
                    None
                }
            })
            .flat_map(choice_values_from_assignment)
            .collect()
    }
}
