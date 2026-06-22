//! Bounded Flowhub service worker helpers for qianji-server.
//!
//! qianji-server owns concrete HTTP/checkpoint state. The dependency-safe
//! worker-loop algorithm lives in `xiuxian-qianji-runtime`.

use std::io;

use xiuxian_qianji_bpmn_engine::BpmnHostBridge;
use xiuxian_qianji_control::{ControlLedger, HotStateStore};
use xiuxian_qianji_runtime::{
    FlowhubServiceWorkerLoopOutput, FlowhubServiceWorkerLoopRequest,
    FlowhubServiceWorkerLoopRuntime, FlowhubServiceWorkerStepOutput,
    run_flowhub_service_worker_completion_loop,
};

use crate::bpmn::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowTaskCompleteReport,
};

/// qianji-server request for one bounded Flowhub service worker loop.
pub type QianjiServerFlowhubServiceWorkerLoopRequest<'a> =
    FlowhubServiceWorkerLoopRequest<'a, QianjiBpmnWorkflowCheckpointBackend>;

/// qianji-server result of one bounded Flowhub service worker loop.
pub type QianjiServerFlowhubServiceWorkerLoopOutput =
    FlowhubServiceWorkerLoopOutput<QianjiBpmnWorkflowTaskCompleteReport>;

/// qianji-server result of one completed Flowhub BPMN service task.
pub type QianjiServerFlowhubServiceWorkerStepOutput = FlowhubServiceWorkerStepOutput;

/// Runs a bounded qianji-server Flowhub service worker loop.
///
/// # Errors
///
/// Returns an I/O error when runtime worker-loop scheduling, activity
/// lifecycle recording, hot-state leasing, or workflow-control completion
/// fails.
pub async fn run_qianji_server_flowhub_service_worker_completion_loop<L, H, B>(
    state: &QianjiBpmnWorkflowHttpState<B>,
    ledger: &L,
    hot_state: &H,
    request: &QianjiServerFlowhubServiceWorkerLoopRequest<'_>,
) -> io::Result<QianjiServerFlowhubServiceWorkerLoopOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
    B: BpmnHostBridge + Send + Sync,
{
    run_flowhub_service_worker_completion_loop(
        FlowhubServiceWorkerLoopRuntime {
            control_port: &state.service,
            host: &state.host,
            ledger,
            hot_state,
        },
        request,
    )
    .await
}
