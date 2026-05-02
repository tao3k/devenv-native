//! Telegram channel authorization policy implementation.

use crate::channels::control_command_authorization::ControlCommandAuthRule;

use super::admin_rules::TelegramCommandAdminRule;

/// Authorization inputs for privileged Telegram control commands.
#[derive(Debug, Clone, Default)]
pub struct TelegramControlCommandPolicy {
    /// Fallback admin identities for control/slash authorization.
    pub admin_users: Vec<String>,
    /// Optional global control-command allow list.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Command-scoped control command rules.
    pub control_command_rules: Vec<TelegramCommandAdminRule>,
    /// Slash-command ACL policy.
    pub slash_command_policy: TelegramSlashCommandPolicy,
}

impl TelegramControlCommandPolicy {
    /// Build control policy from admin identities and command rules.
    #[must_use]
    pub fn new(
        admin_users: Vec<String>,
        control_command_allow_from: Option<Vec<String>>,
        control_command_rules: Vec<TelegramCommandAdminRule>,
    ) -> Self {
        Self {
            admin_users,
            control_command_allow_from,
            control_command_rules,
            slash_command_policy: TelegramSlashCommandPolicy::default(),
        }
    }

    /// Attach slash-command ACL policy to this control policy.
    #[must_use]
    pub fn with_slash_command_policy(
        mut self,
        slash_command_policy: TelegramSlashCommandPolicy,
    ) -> Self {
        self.slash_command_policy = slash_command_policy;
        self
    }
}

/// User-friendly ACL fields for non-privileged Telegram slash commands.
///
/// Priority order:
/// 1) `global` (global override for all listed slash scopes)
/// 2) command-specific allowlists
/// 3) fallback `admin_users` from [`TelegramControlCommandPolicy`]
#[derive(Debug, Clone, Default)]
pub struct TelegramSlashCommandPolicy {
    /// Global slash-command allow list fallback.
    pub global: Option<Vec<String>>,
    /// Allow list for `session.status`.
    pub session_status: Option<Vec<String>>,
    /// Allow list for `session.budget`.
    pub session_budget: Option<Vec<String>>,
    /// Allow list for `session.memory`.
    pub session_memory: Option<Vec<String>>,
    /// Allow list for `session.feedback`.
    pub session_feedback: Option<Vec<String>>,
    /// Allow list for `job.status`.
    pub job_status: Option<Vec<String>>,
    /// Allow list for `jobs.summary`.
    pub jobs_summary: Option<Vec<String>>,
    /// Allow list for `background.submit`.
    pub background_submit: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TelegramSlashCommandRule {
    pub(super) command_scope: &'static str,
    pub(super) allowed_identities: Vec<String>,
}

impl TelegramSlashCommandRule {
    pub(super) fn new(command_scope: &'static str, allowed_identities: Vec<String>) -> Self {
        Self {
            command_scope,
            allowed_identities,
        }
    }
}

impl ControlCommandAuthRule for TelegramSlashCommandRule {
    fn matches(&self, command_text: &str) -> bool {
        self.command_scope == command_text
    }

    fn allows_identity(&self, identity: &str) -> bool {
        self.allowed_identities
            .iter()
            .any(|entry| entry == "*" || entry == identity)
    }
}
