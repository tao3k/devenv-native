use super::markdown::markdown_workflow_tasks;
use super::render::render_workflow_bpmn_xml;
use crate::bpmn::http_transport::QianjiControlWorkflowSourceAdmissionHttpRequest;

pub(super) const MARKDOWN_WORKFLOW_COMPILER: &str = "qianji-server-markdown-step-compiler-v1";
pub(super) const MARKDOWN_MEDIA_TYPE: &str = "text/markdown";

pub(super) struct WorkflowSourceCompilation {
    pub bpmn_xml: String,
    pub compiler: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkflowSourceCompileError {
    MarkdownStepsMissing,
}

pub(super) fn compile_markdown_workflow_source(
    request: &QianjiControlWorkflowSourceAdmissionHttpRequest,
) -> Result<WorkflowSourceCompilation, WorkflowSourceCompileError> {
    let source_id = xml_id(&request.source_id);
    let process_id = request.process_id.as_str();
    let workflow_name = request.workflow_name.trim();
    let workflow_description = request.workflow_description.trim();
    let tasks = markdown_workflow_tasks(
        workflow_name,
        workflow_description,
        request.source_text.as_str(),
    )?;
    Ok(WorkflowSourceCompilation {
        bpmn_xml: render_workflow_bpmn_xml(source_id.as_str(), process_id, workflow_name, &tasks),
        compiler: MARKDOWN_WORKFLOW_COMPILER,
    })
}

pub(super) fn xml_id(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => character,
            _ => '_',
        })
        .collect()
}

pub(super) fn xml_attr(value: &str) -> String {
    xml_escape(value)
}

pub(super) fn xml_text(value: &str) -> String {
    xml_escape(value)
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
