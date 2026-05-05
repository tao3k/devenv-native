use tokio::sync::mpsc;

use crate::channels::discord::runtime::foreground::DiscordForegroundRuntime;
use crate::channels::discord::runtime::telemetry::emit_runtime_snapshot;
use crate::channels::traits::ChannelMessage;
use crate::jobs::JobCompletion;

pub(super) async fn drive_ingress_runtime_loop(
    runtime: &mut DiscordForegroundRuntime,
    inbound_rx: &mut mpsc::Receiver<ChannelMessage>,
    completion_rx: &mut mpsc::Receiver<JobCompletion>,
    inbound_snapshot_tx: &mpsc::Sender<ChannelMessage>,
    inbound_queue_capacity: usize,
    snapshot_tick: &mut Option<tokio::time::Interval>,
    ingress_server: &mut tokio::task::JoinHandle<std::io::Result<()>>,
) {
    loop {
        tokio::select! {
            maybe_msg = inbound_rx.recv() => {
                let Some(msg) = maybe_msg else {
                    break;
                };
                runtime.spawn_foreground_turn(msg).await;
            }
            maybe_completion = completion_rx.recv() => {
                let Some(completion) = maybe_completion else {
                    continue;
                };
                runtime.push_completion(completion).await;
            }
            () = runtime.join_next_foreground_task(), if runtime.has_foreground_tasks() => {
            }
            () = async {
                if let Some(interval) = snapshot_tick.as_mut() {
                    let _ = interval.tick().await;
                }
            }, if snapshot_tick.is_some() => {
                let foreground_snapshot = runtime.snapshot();
                let admission_snapshot = runtime.admission_runtime_snapshot();
                emit_runtime_snapshot(
                    "ingress",
                    inbound_snapshot_tx,
                    inbound_queue_capacity,
                    &foreground_snapshot,
                    admission_snapshot,
                );
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
            result = &mut *ingress_server => {
                match result {
                    Ok(Ok(())) => tracing::warn!("discord ingress server exited"),
                    Ok(Err(error)) => tracing::error!("discord ingress server failed: {error}"),
                    Err(error) => tracing::error!("discord ingress task join error: {error}"),
                }
                break;
            }
        }
    }
}
