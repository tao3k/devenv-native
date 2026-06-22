use super::contract::{
    MARKDOWN_WORKFLOW_COMPILER, WorkflowSourceCompilation, WorkflowSourceCompileError, xml_id,
};
use super::markdown::markdown_workflow_tasks;
use super::render::render_workflow_bpmn_xml;
use crate::bpmn::http_transport::QianjiControlWorkflowSourceAdmissionHttpRequest;

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
