use super::{BpmnSourceFile, Event, LintIssue, Reader, issue_for_tag, local_name};

pub(in crate::lint::bpmn) fn deferred_document_surface_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut stack = Vec::<String>::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let name = event.name();
                let tag = local_name(name.as_ref())?;
                let parent = stack.last().map(String::as_str);
                if let Some(issue) = issue_for_tag(source, tag, parent) {
                    return Some(issue);
                }
                stack.push(tag.to_string());
            }
            Ok(Event::Empty(event)) => {
                let name = event.name();
                let tag = local_name(name.as_ref())?;
                let parent = stack.last().map(String::as_str);
                if let Some(issue) = issue_for_tag(source, tag, parent) {
                    return Some(issue);
                }
            }
            Ok(Event::End(_)) => {
                let _ = stack.pop();
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}
