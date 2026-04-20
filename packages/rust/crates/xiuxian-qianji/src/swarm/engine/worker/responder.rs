use crate::scheduler::QianjiScheduler;
use crate::swarm::engine_orchestrator::SwarmEngine;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::Duration;

impl SwarmEngine {
    pub(in crate::swarm::engine::worker) fn start_remote_responder(
        scheduler: Arc<QianjiScheduler>,
        role: Option<String>,
        agent_id: String,
        remote_enabled: bool,
        poll_interval_ms: u64,
    ) -> (
        Option<watch::Sender<bool>>,
        Option<tokio::task::JoinHandle<()>>,
    ) {
        let Some(role_class) = role else {
            return (None, None);
        };
        if !remote_enabled {
            return (None, None);
        }

        let (stop_tx, mut stop_rx) = watch::channel(false);
        let interval = Duration::from_millis(poll_interval_ms);
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                    result = scheduler.process_remote_possession_once(
                        role_class.as_str(),
                        agent_id.as_str(),
                        interval,
                    ) => {
                        if let Err(error) = result {
                            log::warn!("remote possession responder failed for {agent_id}: {error}");
                        }
                    }
                }
            }
        });
        (Some(stop_tx), Some(handle))
    }

    pub(in crate::swarm::engine::worker) async fn stop_remote_responder(
        stop_tx: Option<watch::Sender<bool>>,
        responder_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        if let Some(stop_tx) = stop_tx {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = responder_handle {
            let _ = handle.await;
        }
    }
}
