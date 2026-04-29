use super::{
    ActiveTask, BpmnSourceFile, BytesStart, Event, HashMap, ProcessMetadata, Reader,
    SequenceFlowMetadata, TaskAssociationCapture, TaskAssociationContext, append_entity_reference,
    attribute_value, is_span_only_node_tag, is_task_tag, local_name, parse_variable_names,
    reader_position, record_node_span, start_or_empty_event_span,
};

pub(super) fn collect_process_metadata(
    source: &BpmnSourceFile,
) -> HashMap<String, ProcessMetadata> {
    MetadataCollector::new(source).collect()
}

pub(super) struct MetadataCollector<'a> {
    source: &'a BpmnSourceFile,
    processes: HashMap<String, ProcessMetadata>,
    active_process_id: Option<String>,
    active_metadata: ProcessMetadata,
    active_task: Option<ActiveTask>,
}

impl<'a> MetadataCollector<'a> {
    fn new(source: &'a BpmnSourceFile) -> Self {
        Self {
            source,
            processes: HashMap::new(),
            active_process_id: None,
            active_metadata: ProcessMetadata::default(),
            active_task: None,
        }
    }

    fn collect(mut self) -> HashMap<String, ProcessMetadata> {
        let mut reader = Reader::from_str(&self.source.contents);
        reader.config_mut().trim_text(false);
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => self.handle_start(&reader, &event),
                Ok(Event::Empty(event)) => self.handle_empty(&reader, &event),
                Ok(Event::Text(event)) => {
                    if let Some(task) = self.active_task.as_mut()
                        && let Ok(text) = event.decode()
                    {
                        append_task_association_variables(task, &text);
                    }
                }
                Ok(Event::GeneralRef(event)) => self.handle_general_ref(&event),
                Ok(Event::End(event)) => self.handle_end(event.name().as_ref()),
                Ok(Event::Eof) | Err(_) => return self.processes,
                Ok(_) => {}
            }
        }
    }

    fn handle_start(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "process" => self.start_process(reader, event),
            tag if is_task_tag(tag) => self.start_task(reader, event),
            "sequenceFlow" => self.record_sequence_flow(reader, event),
            tag if is_span_only_node_tag(tag) => self.record_span(reader, event),
            "dataInput" => {
                self.record_active_task_io_span(reader, event, true);
                if let Some(task) = self.active_task.as_mut()
                    && let Some(name) = attribute_value(reader, event, "name")
                {
                    task.inputs.insert(name);
                }
            }
            "dataOutput" => {
                self.record_active_task_io_span(reader, event, false);
                if let Some(task) = self.active_task.as_mut()
                    && let Some(name) = attribute_value(reader, event, "name")
                {
                    task.outputs.insert(name);
                }
            }
            "dataInputAssociation" => {
                self.record_active_task_io_span(reader, event, true);
                if let Some(task) = self.active_task.as_mut() {
                    task.association_context = Some(TaskAssociationContext::Input);
                }
            }
            "sourceRef" => {
                if let Some(task) = self.active_task.as_mut()
                    && task.association_context == Some(TaskAssociationContext::Input)
                {
                    task.association_capture = Some(TaskAssociationCapture::InputSourceRef);
                }
            }
            "dataOutputAssociation" => {
                self.record_active_task_io_span(reader, event, false);
                if let Some(task) = self.active_task.as_mut() {
                    task.association_context = Some(TaskAssociationContext::Output);
                }
            }
            "targetRef" => {
                if let Some(task) = self.active_task.as_mut()
                    && task.association_context == Some(TaskAssociationContext::Output)
                {
                    task.association_capture = Some(TaskAssociationCapture::OutputTargetRef);
                }
            }
            _ => {}
        }
    }

    fn handle_empty(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "dataInput" => {
                self.record_active_task_io_span(reader, event, true);
                if let Some(task) = self.active_task.as_mut()
                    && let Some(name) = attribute_value(reader, event, "name")
                {
                    task.inputs.insert(name);
                }
            }
            "dataOutput" => {
                self.record_active_task_io_span(reader, event, false);
                if let Some(task) = self.active_task.as_mut()
                    && let Some(name) = attribute_value(reader, event, "name")
                {
                    task.outputs.insert(name);
                }
            }
            "sequenceFlow" => self.record_sequence_flow(reader, event),
            tag if is_task_tag(tag) => self.record_empty_task(reader, event),
            tag if is_span_only_node_tag(tag) => self.record_span(reader, event),
            _ => {}
        }
    }

    fn handle_general_ref(&mut self, event: &quick_xml::events::BytesRef<'_>) {
        if let Some(task) = self.active_task.as_mut() {
            let reference = event.decode().ok();
            let mut text = String::new();
            append_entity_reference(&mut text, reference.as_deref());
            append_task_association_variables(task, &text);
        }
    }

    fn handle_end(&mut self, raw_name: &[u8]) {
        let name = local_name(raw_name);
        match name.as_str() {
            "sourceRef" | "targetRef" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.association_capture = None;
                }
            }
            "dataInputAssociation" | "dataOutputAssociation" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.association_context = None;
                }
            }
            tag if is_task_tag(tag) => self.finish_task(),
            "process" => self.finish_process(),
            _ => {}
        }
    }

    fn start_process(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        self.active_process_id = attribute_value(reader, event, "id");
        self.active_metadata = ProcessMetadata::default();
    }

    fn start_task(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        self.record_span_for_id(reader, event, &id);
        self.active_task = Some(ActiveTask {
            id,
            ..ActiveTask::default()
        });
    }

    fn record_empty_task(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        self.record_span_for_id(reader, event, &id);
        self.active_metadata
            .task_inputs
            .entry(id.clone())
            .or_default();
        self.active_metadata.task_outputs.entry(id).or_default();
    }

    fn record_span(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        if let Some(id) = attribute_value(reader, event, "id") {
            self.record_span_for_id(reader, event, &id);
        }
    }

    fn record_sequence_flow(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        if attribute_value(reader, event, "sourceRef").is_none() {
            return;
        }
        let Some(target_ref) = attribute_value(reader, event, "targetRef") else {
            return;
        };
        let Some(event_end) = reader_position(reader) else {
            return;
        };
        let Some(span) = start_or_empty_event_span(&self.source.contents, event_end, event) else {
            return;
        };
        self.active_metadata
            .sequence_flows
            .insert(id, SequenceFlowMetadata { target_ref, span });
    }

    fn record_span_for_id(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>, id: &str) {
        record_node_span(
            &mut self.active_metadata,
            &self.source.contents,
            reader,
            event,
            id,
        );
        if let Some(default_flow) = attribute_value(reader, event, "default") {
            self.active_metadata
                .gateway_default_flows
                .insert(id.to_string(), default_flow);
        }
    }

    fn record_active_task_io_span(
        &mut self,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_input: bool,
    ) {
        let Some(task_id) = self.active_task.as_ref().map(|task| task.id.clone()) else {
            return;
        };
        let Some(event_end) = reader_position(reader) else {
            return;
        };
        let Some(span) = start_or_empty_event_span(&self.source.contents, event_end, event) else {
            return;
        };
        let spans = if is_input {
            &mut self.active_metadata.task_input_spans
        } else {
            &mut self.active_metadata.task_output_spans
        };
        spans.insert(task_id, span);
    }

    fn finish_task(&mut self) {
        if let Some(task) = self.active_task.take() {
            self.active_metadata
                .task_inputs
                .insert(task.id.clone(), task.inputs);
            self.active_metadata
                .task_outputs
                .insert(task.id, task.outputs);
        }
    }

    fn finish_process(&mut self) {
        if let Some(process_id) = self.active_process_id.take() {
            self.processes
                .insert(process_id, std::mem::take(&mut self.active_metadata));
        }
    }
}

pub(super) fn append_task_association_variables(task: &mut ActiveTask, text: &str) {
    match task.association_capture {
        Some(TaskAssociationCapture::InputSourceRef) => {
            task.inputs.extend(parse_variable_names(text));
        }
        Some(TaskAssociationCapture::OutputTargetRef) => {
            task.outputs.extend(parse_variable_names(text));
        }
        None => {}
    }
}
