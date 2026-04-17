use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::agent::Agent;
use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::telegram::runtime::jobs::{
    handle_inbound_message_with_interrupt, push_background_completion,
};
use crate::channels::telegram::runtime::telemetry::{
    emit_runtime_snapshot, snapshot_interval_from_env,
};
use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager};

pub(super) struct PollingLoopReceivers<'a> {
    pub inbound_rx: &'a mut mpsc::Receiver<ChannelMessage>,
    pub completion_rx: &'a mut mpsc::Receiver<JobCompletion>,
}

pub(super) struct PollingLoopContext<'a> {
    pub inbound_tx: &'a mpsc::Sender<ChannelMessage>,
    pub channel_for_send: &'a Arc<dyn Channel>,
    pub foreground_tx: &'a mpsc::Sender<ChannelMessage>,
    pub interrupt_controller: &'a ForegroundInterruptController,
    pub job_manager: &'a Arc<JobManager>,
    pub agent: &'a Arc<Agent>,
    pub runtime_config: TelegramRuntimeConfig,
}

pub(super) async fn run_polling_event_loop(
    PollingLoopReceivers {
        inbound_rx,
        completion_rx,
    }: PollingLoopReceivers<'_>,
    context: PollingLoopContext<'_>,
) {
    let PollingLoopContext {
        inbound_tx,
        channel_for_send,
        foreground_tx,
        interrupt_controller,
        job_manager,
        agent,
        runtime_config,
    } = context;
    let mut snapshot_tick = snapshot_interval_from_env().map(|period| {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval
    });
    if let Some(interval) = snapshot_tick.as_mut() {
        let _ = interval.tick().await;
    }

    loop {
        tokio::select! {
            maybe_msg = inbound_rx.recv() => {
                let Some(msg) = maybe_msg else {
                    break;
                };
                if !handle_inbound_message_with_interrupt(
                    msg,
                    channel_for_send,
                    foreground_tx,
                    interrupt_controller,
                    job_manager,
                    agent,
                    runtime_config.foreground_queue_mode,
                )
                .await {
                    break;
                }
            }
            maybe_completion = completion_rx.recv() => {
                let Some(completion) = maybe_completion else {
                    continue;
                };
                push_background_completion(channel_for_send, agent, completion).await;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
            () = async {
                if let Some(interval) = snapshot_tick.as_mut() {
                    let _ = interval.tick().await;
                }
            }, if snapshot_tick.is_some() => {
                emit_runtime_snapshot(
                    "polling",
                    inbound_tx,
                    foreground_tx,
                    runtime_config,
                    agent.downstream_admission_runtime_snapshot(),
                );
            }
        }
    }
}
