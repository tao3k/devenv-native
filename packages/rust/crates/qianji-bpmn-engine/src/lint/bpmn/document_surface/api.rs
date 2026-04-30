use super::shared::{
    BpmnSourceFile, Event, LintIssue, Reader, diagram_anchor_issue, diagram_anchor_kind_issue,
    diagram_boolean_issue, diagram_completeness_issue, diagram_enum_issue, diagram_identity_issue,
    diagram_namespace_issue, diagram_numeric_issue, diagram_reference_issue,
    diagram_topology_issue, flow_element_metadata_issue, issue_for_tag, local_name,
    resource_role_metadata_issue,
};

pub(in crate::lint::bpmn) fn deferred_document_surface_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    if let Some(issue) = resource_role_metadata_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = flow_element_metadata_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_namespace_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_boolean_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_numeric_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_enum_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_topology_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_anchor_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_identity_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_reference_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_anchor_kind_issue(source) {
        return Some(issue);
    }
    if let Some(issue) = diagram_completeness_issue(source) {
        return Some(issue);
    }

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
