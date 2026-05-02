use std::collections::HashMap;

use super::principals::collect_principals;
use crate::config::DiscordAclSlashSettings;

pub(super) struct SlashOverrides {
    pub(super) command: Option<Vec<String>>,
    pub(super) session_status: Option<Vec<String>>,
    pub(super) session_budget: Option<Vec<String>>,
    pub(super) session_memory: Option<Vec<String>>,
    pub(super) session_feedback: Option<Vec<String>>,
    pub(super) job_status: Option<Vec<String>>,
    pub(super) jobs_summary: Option<Vec<String>>,
    pub(super) background_submit: Option<Vec<String>>,
}

pub(super) fn slash_overrides(
    slash: Option<&DiscordAclSlashSettings>,
    role_aliases: &HashMap<String, String>,
) -> SlashOverrides {
    let Some(slash) = slash else {
        return SlashOverrides {
            command: None,
            session_status: None,
            session_budget: None,
            session_memory: None,
            session_feedback: None,
            job_status: None,
            jobs_summary: None,
            background_submit: None,
        };
    };

    SlashOverrides {
        command: slash
            .global
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        session_status: slash
            .session_status
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        session_budget: slash
            .session_budget
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        session_memory: slash
            .session_memory
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        session_feedback: slash
            .session_feedback
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        job_status: slash
            .job_status
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        jobs_summary: slash
            .jobs_summary
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
        background_submit: slash
            .background_submit
            .as_ref()
            .and_then(|principal| collect_principals(principal, role_aliases)),
    }
}
