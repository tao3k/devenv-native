use super::{
    ActiveNativeInteractionTask, Event, Reader, StaticInteractionChoiceOutput,
    append_entity_reference, attribute_value, is_element, local_name,
};

pub(in crate::lint::bpmn::condition_contract) fn collect_static_interaction_choice_outputs(
    contents: &str,
) -> Vec<StaticInteractionChoiceOutput> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut active_task: Option<ActiveNativeInteractionTask> = None;
    let mut outputs = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                if is_element(&event, "userTask") {
                    active_task = attribute_value(&reader, &event, "id")
                        .map(ActiveNativeInteractionTask::new);
                } else if let Some(task) = active_task.as_mut() {
                    task.handle_start(&reader, &event);
                }
            }
            Ok(Event::Empty(event)) => {
                if let Some(task) = active_task.as_mut() {
                    task.handle_empty(&reader, &event);
                }
            }
            Ok(Event::Text(event)) => {
                if let Some(task) = active_task.as_mut()
                    && let Ok(text) = event.decode()
                {
                    task.append_text(&text);
                }
            }
            Ok(Event::GeneralRef(event)) => {
                if let Some(task) = active_task.as_mut() {
                    let reference = event.decode().ok();
                    let mut text = String::new();
                    append_entity_reference(&mut text, reference.as_deref());
                    task.append_text(&text);
                }
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "userTask" {
                    if let Some(output) = active_task
                        .take()
                        .and_then(ActiveNativeInteractionTask::finish_output)
                    {
                        outputs.push(output);
                    }
                } else if let Some(task) = active_task.as_mut() {
                    task.handle_end(name);
                }
            }
            Ok(Event::Eof) | Err(_) => return outputs,
            Ok(_) => {}
        }
    }
}
