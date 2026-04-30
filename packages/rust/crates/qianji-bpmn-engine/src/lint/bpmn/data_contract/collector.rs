use super::{
    BpmnSourceFile, BytesStart, Event, HashSet, ProcessContract, Range, Reader,
    append_entity_reference, attribute_value, is_task_tag, local_name, parse_output_names,
    reader_position, start_event_span,
};

#[derive(Default)]
pub(super) struct SequenceFlowContract {
    pub(super) source_ref: String,
    pub(super) target_ref: String,
    pub(super) condition: Option<String>,
    pub(super) condition_span: Option<Range<usize>>,
}

#[derive(Default)]
pub(super) struct ActiveTask {
    id: String,
    outputs: HashSet<String>,
    in_output_association: bool,
    in_output_target_ref: bool,
}

#[derive(Default)]
pub(super) struct ActiveFlow {
    source_ref: String,
    target_ref: String,
    condition: String,
    condition_span: Option<Range<usize>>,
    in_condition: bool,
}

pub(super) fn collect_process_contracts(source: &BpmnSourceFile) -> Vec<ProcessContract> {
    ProcessContractCollector::new(source).collect()
}

pub(super) struct ProcessContractCollector<'a> {
    source: &'a BpmnSourceFile,
    processes: Vec<ProcessContract>,
    active_process: Option<ProcessContract>,
    active_task: Option<ActiveTask>,
    active_flow: Option<ActiveFlow>,
}

impl<'a> ProcessContractCollector<'a> {
    fn new(source: &'a BpmnSourceFile) -> Self {
        Self {
            source,
            processes: Vec::new(),
            active_process: None,
            active_task: None,
            active_flow: None,
        }
    }

    fn collect(mut self) -> Vec<ProcessContract> {
        let mut reader = Reader::from_str(&self.source.contents);
        reader.config_mut().trim_text(false);
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => self.handle_start(&reader, &event),
                Ok(Event::Empty(event)) => self.handle_empty(&reader, &event),
                Ok(Event::Text(event)) => self.handle_text(event.decode().ok().as_deref()),
                Ok(Event::CData(event)) => {
                    self.handle_condition_text(event.decode().ok().as_deref());
                }
                Ok(Event::GeneralRef(event)) => {
                    let reference = event.decode().ok();
                    let mut text = String::new();
                    append_entity_reference(&mut text, reference.as_deref());
                    self.handle_condition_text(Some(&text));
                }
                Ok(Event::End(event)) => self.handle_end(event.name().as_ref()),
                Ok(Event::Eof) | Err(_) => return self.processes,
                Ok(_) => {}
            }
        }
    }

    fn handle_start(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "process" => {
                self.active_process = Some(ProcessContract {
                    id: attribute_value(reader, event, "id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    ..ProcessContract::default()
                });
            }
            tag if is_task_tag(tag) && self.active_process.is_some() => {
                self.active_task = Some(ActiveTask {
                    id: attribute_value(reader, event, "id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    ..ActiveTask::default()
                });
            }
            "exclusiveGateway" => self.record_gateway(reader, event),
            "sequenceFlow" if self.active_process.is_some() => {
                self.active_flow = Some(ActiveFlow {
                    source_ref: attribute_value(reader, event, "sourceRef").unwrap_or_default(),
                    target_ref: attribute_value(reader, event, "targetRef").unwrap_or_default(),
                    ..ActiveFlow::default()
                });
            }
            "dataOutput" => self.record_task_output(reader, event),
            "dataOutputAssociation" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_association = true;
                }
            }
            "targetRef" => {
                if let Some(task) = self.active_task.as_mut()
                    && task.in_output_association
                {
                    task.in_output_target_ref = true;
                }
            }
            "conditionExpression" => self.start_condition(reader, event),
            _ => {}
        }
    }

    fn handle_empty(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        match local_name(event.name().as_ref()).as_str() {
            "exclusiveGateway" => self.record_gateway(reader, event),
            "dataOutput" => self.record_task_output(reader, event),
            "sequenceFlow" => {
                if let Some(process) = self.active_process.as_mut() {
                    process.flows.push(SequenceFlowContract {
                        source_ref: attribute_value(reader, event, "sourceRef").unwrap_or_default(),
                        target_ref: attribute_value(reader, event, "targetRef").unwrap_or_default(),
                        ..SequenceFlowContract::default()
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: Option<&str>) {
        if let Some(task) = self.active_task.as_mut()
            && task.in_output_target_ref
            && let Some(text) = text
        {
            task.outputs.extend(parse_output_names(text));
        }
        self.handle_condition_text(text);
    }

    fn handle_condition_text(&mut self, text: Option<&str>) {
        if let Some(flow) = self.active_flow.as_mut()
            && flow.in_condition
            && let Some(text) = text
        {
            flow.condition.push_str(text);
        }
    }

    fn handle_end(&mut self, raw_name: &[u8]) {
        let name = local_name(raw_name);
        match name.as_str() {
            "targetRef" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_target_ref = false;
                }
            }
            "dataOutputAssociation" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_association = false;
                }
            }
            "conditionExpression" => {
                if let Some(flow) = self.active_flow.as_mut() {
                    flow.in_condition = false;
                }
            }
            "sequenceFlow" => self.finish_flow(),
            tag if is_task_tag(tag) => self.finish_task(),
            "process" => {
                if let Some(process) = self.active_process.take() {
                    self.processes.push(process);
                }
            }
            _ => {}
        }
    }

    fn record_gateway(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let (Some(process), Some(id)) = (
            self.active_process.as_mut(),
            attribute_value(reader, event, "id"),
        ) {
            process.gateways.insert(id);
        }
    }

    fn record_task_output(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(task) = self.active_task.as_mut()
            && let Some(name) = attribute_value(reader, event, "name")
        {
            task.outputs.insert(name);
        }
    }

    fn start_condition(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(flow) = self.active_flow.as_mut() {
            flow.in_condition = true;
            flow.condition.clear();
            flow.condition_span =
                reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        }
    }

    fn finish_flow(&mut self) {
        if let (Some(process), Some(flow)) = (self.active_process.as_mut(), self.active_flow.take())
        {
            process.flows.push(SequenceFlowContract {
                source_ref: flow.source_ref,
                target_ref: flow.target_ref,
                condition: (!flow.condition.trim().is_empty())
                    .then(|| flow.condition.trim().to_string()),
                condition_span: flow.condition_span,
            });
        }
    }

    fn finish_task(&mut self) {
        if let (Some(process), Some(task)) = (self.active_process.as_mut(), self.active_task.take())
        {
            process.task_outputs.insert(task.id, task.outputs);
        }
    }
}
