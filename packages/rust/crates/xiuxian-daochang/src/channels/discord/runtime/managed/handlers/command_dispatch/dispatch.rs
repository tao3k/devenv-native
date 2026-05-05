use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_runtime::turn::build_session_id;
use crate::channels::traits::{Channel, ChannelMessage};
use crate::jobs::JobManager;

use super::{control, jobs, session};
use crate::channels::discord::runtime::managed::parsing::{ManagedCommand, parse_managed_command};

pub(crate) async fn handle_inbound_managed_command(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    job_manager: &Arc<JobManager>,
) -> bool {
    let Some(command) = parse_managed_command(&msg.content) else {
        return false;
    };
    let session_id = build_session_id(&msg.channel, &msg.session_key);

    match command {
        ManagedCommand::Help(format) => {
            control::handle_help(channel, msg, format).await;
            true
        }
        ManagedCommand::Reset => {
            control::handle_reset(agent, channel, msg, &session_id).await;
            true
        }
        ManagedCommand::Resume(resume_command) => {
            control::handle_resume(agent, channel, msg, &session_id, resume_command).await;
            true
        }
        ManagedCommand::SessionStatus(format) => {
            session::handle_session_status(agent, channel, msg, &session_id, format).await;
            true
        }
        ManagedCommand::SessionBudget(format) => {
            session::handle_session_budget(agent, channel, msg, &session_id, format).await;
            true
        }
        ManagedCommand::SessionMemory(format) => {
            session::handle_session_memory(agent, channel, msg, &session_id, format).await;
            true
        }
        ManagedCommand::SessionAdmin(command) => {
            session::handle_session_admin(channel, msg, command).await;
            true
        }
        ManagedCommand::SessionFeedback(command) => {
            session::handle_session_feedback(agent, channel, msg, &session_id, command).await;
            true
        }
        ManagedCommand::SessionInjection(command) => {
            session::handle_session_injection(agent, channel, msg, &session_id, command).await;
            true
        }
        ManagedCommand::SessionMention(command) => {
            session::handle_session_mention(channel, msg, command).await;
            true
        }
        ManagedCommand::SessionPartition(command) => {
            session::handle_session_partition(channel, msg, command).await;
            true
        }
        ManagedCommand::JobStatus { job_id, format } => {
            jobs::handle_job_status(channel, msg, job_manager, job_id, format).await;
            true
        }
        ManagedCommand::JobsSummary(format) => {
            jobs::handle_jobs_summary(channel, msg, job_manager, format).await;
            true
        }
        ManagedCommand::BackgroundSubmit(prompt) => {
            jobs::handle_background_submit(channel, msg, job_manager, &session_id, prompt).await;
            true
        }
    }
}
