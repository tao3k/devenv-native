use super::{
    BpmnSourceFile, BytesStart, CallActivityContext, GlobalTaskContext, HumanTaskContext,
    LintIssue, ProcessContext, Reader, attribute_value, event_span, is_assignment_role,
    is_global_task, is_human_interaction_task, is_unsupported_assignment_role, local_name,
    native_rendering_issue, unsupported_assignment_child_issue,
    unsupported_assignment_semantics_issue, unsupported_global_task_binding_issue,
};

#[derive(Default)]
pub(super) struct HumanTaskStandardScanState {
    active_tasks: Vec<HumanTaskContext>,
    active_roles: Vec<String>,
    active_processes: Vec<ProcessContext>,
    global_tasks: Vec<GlobalTaskContext>,
    call_activities: Vec<CallActivityContext>,
}

impl HumanTaskStandardScanState {
    pub(super) fn handle_start(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
        is_empty: bool,
    ) {
        let tag = local_name(event.name().as_ref()).to_string();

        if tag == "process" {
            if !is_empty {
                self.active_processes.push(ProcessContext {
                    process_id: attribute_value(reader, event, "id"),
                });
            }
            return;
        }

        if tag == "callActivity"
            && let (Some(process), Some(called_element)) = (
                self.active_processes.last(),
                attribute_value(reader, event, "calledElement"),
            )
        {
            self.call_activities.push(CallActivityContext {
                process_id: process.process_id.clone(),
                activity_id: attribute_value(reader, event, "id"),
                called_element,
                span: event_span(reader, event).unwrap_or(0..0),
            });
        }

        if is_global_task(&tag)
            && self.active_processes.is_empty()
            && let Some(task_id) = attribute_value(reader, event, "id")
        {
            self.global_tasks.push(GlobalTaskContext {
                task_id,
                task_kind: tag.clone(),
            });
        }

        if is_human_interaction_task(&tag) {
            let context = HumanTaskContext {
                task_id: attribute_value(reader, event, "id"),
                task_kind: tag.clone(),
            };
            if !is_empty {
                self.active_tasks.push(context);
            }
            return;
        }

        if tag == "rendering" {
            if let Some(task) = self.active_tasks.last() {
                issues.push(native_rendering_issue(source, reader, event, task));
            }
            return;
        }

        if is_assignment_role(&tag) {
            if let Some(task) = self.active_tasks.last()
                && is_unsupported_assignment_role(&tag)
            {
                issues.push(unsupported_assignment_semantics_issue(
                    source, reader, event, task, &tag,
                ));
            }
            if !is_empty {
                self.active_roles.push(tag);
            }
            return;
        }

        if matches!(tag.as_str(), "resourceParameterBinding" | "participantRef")
            && let (Some(task), Some(role)) = (self.active_tasks.last(), self.active_roles.last())
        {
            issues.push(unsupported_assignment_child_issue(
                source, reader, event, task, role, &tag,
            ));
        }
    }

    pub(super) fn handle_end(&mut self, event: &quick_xml::events::BytesEnd<'_>) {
        let name = event.name();
        let tag = local_name(name.as_ref());
        if tag == "process" {
            self.active_processes.pop();
        } else if is_human_interaction_task(tag) {
            self.active_tasks.pop();
        } else if is_assignment_role(tag) {
            self.active_roles.pop();
        }
    }

    pub(super) fn global_task_binding_issues(&self, source: &BpmnSourceFile) -> Vec<LintIssue> {
        self.call_activities
            .iter()
            .filter_map(|call_activity| {
                self.global_tasks
                    .iter()
                    .find(|task| task.task_id == call_activity.called_element)
                    .map(|task| unsupported_global_task_binding_issue(source, call_activity, task))
            })
            .collect()
    }
}
