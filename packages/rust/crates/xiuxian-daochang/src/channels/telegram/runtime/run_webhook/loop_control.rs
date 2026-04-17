use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::agent::Agent;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::telegram::runtime::jobs::{
    handle_inbound_message_with_interrupt, push_background_completion,
};
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::{JobCompletion, JobManager};

use super::server::drain_finished_webhook_server;

pub(super) struct WebhookLoopReceivers<'a> {
    pub inbound_rx: &'a mut mpsc::Receiver<ChannelMessage>,
    pub completion_rx: &'a mut mpsc::Receiver<JobCompletion>,
}

pub(super) struct WebhookLoopContext<'a> {
    pub channel_for_send: &'a Arc<dyn Channel>,
    pub foreground_tx: &'a mpsc::Sender<ChannelMessage>,
    pub interrupt_controller: &'a ForegroundInterruptController,
    pub job_manager: &'a Arc<JobManager>,
    pub agent: &'a Arc<Agent>,
    pub foreground_queue_mode: ForegroundQueueMode,
    pub webhook_server: &'a mut tokio::task::JoinHandle<std::io::Result<()>>,
}

pub(super) async fn run_webhook_event_loop(
    WebhookLoopReceivers {
        inbound_rx,
        completion_rx,
    }: WebhookLoopReceivers<'_>,
    context: WebhookLoopContext<'_>,
) {
    let WebhookLoopContext {
        channel_for_send,
        foreground_tx,
        interrupt_controller,
        job_manager,
        agent,
        foreground_queue_mode,
        webhook_server,
    } = context;
    let mut health_tick = tokio::time::interval(Duration::from_secs(1));
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
                    foreground_queue_mode,
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
            _ = health_tick.tick() => {
                if drain_finished_webhook_server(webhook_server).await {
                    break;
                }
            }
        }
    }
}
