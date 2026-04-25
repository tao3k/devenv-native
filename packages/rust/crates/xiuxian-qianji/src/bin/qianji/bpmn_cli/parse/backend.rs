use crate::bpmn_cli::deps::QianjiBpmnWorkflowCheckpointBackend;

pub(super) fn parse_bpmn_checkpoint_backend(
    checkpoint_runtime: bool,
) -> Option<QianjiBpmnWorkflowCheckpointBackend> {
    if checkpoint_runtime {
        Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)
    } else {
        #[cfg(feature = "duckdb")]
        {
            Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb)
        }
        #[cfg(not(feature = "duckdb"))]
        {
            None
        }
    }
}
