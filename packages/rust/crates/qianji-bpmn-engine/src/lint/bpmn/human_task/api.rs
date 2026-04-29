use super::{
    BpmnEngineError, BpmnSourceFile, Event, HumanTaskStandardScanState, LintIssue, Reader,
};

pub(in crate::lint::bpmn) fn human_task_standard_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut state = HumanTaskStandardScanState::default();
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                state.handle_start(source, &reader, &event, &mut issues, false);
            }
            Ok(Event::Empty(event)) => {
                state.handle_start(source, &reader, &event, &mut issues, true);
            }
            Ok(Event::End(event)) => state.handle_end(&event),
            Ok(Event::Eof) => {
                issues.extend(state.global_human_task_binding_issues(source));
                return issues;
            }
            Err(_) => return issues,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn) fn issue_from_bpmn_human_task_standard_error(
    source: &BpmnSourceFile,
    error: &BpmnEngineError,
) -> Option<LintIssue> {
    if let BpmnEngineError::UnknownCalledProcess {
        process_id,
        node_id,
        called_process_id,
    } = error
    {
        return human_task_standard_issues(source)
            .into_iter()
            .find(|issue| {
                issue.code == "bpmn.unsupported_global_human_task_binding"
                    && issue.evidence["process_id"].as_str() == Some(process_id.as_str())
                    && issue.evidence["call_activity_id"].as_str() == Some(node_id.as_str())
                    && issue.evidence["called_element"].as_str() == Some(called_process_id.as_str())
            });
    }

    let BpmnEngineError::UnsupportedElement { element, .. } = error else {
        return None;
    };
    if !matches!(
        element.as_str(),
        "rendering" | "performer" | "resourceRole" | "participantRef" | "resourceParameterBinding"
    ) {
        return None;
    }
    human_task_standard_issues(source)
        .into_iter()
        .find(|issue| issue.evidence["element"].as_str() == Some(element.as_str()))
}
