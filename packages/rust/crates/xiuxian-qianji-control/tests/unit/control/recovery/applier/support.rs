use std::error::Error;

use xiuxian_qianji_control::{WorkerId, WorkerRef};

pub(super) fn worker_ref() -> Result<WorkerRef, Box<dyn Error>> {
    Ok(WorkerRef {
        worker_id: WorkerId::new("worker-recovery-applier")?,
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    })
}
