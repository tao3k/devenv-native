//! Scheduler execution identity for role-aware distributed routing.

/// Execution identity used by scheduler-level node ownership checks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulerAgentIdentity {
    /// Optional stable agent id, usually from `AGENT_ID`.
    pub agent_id: Option<String>,
    /// Optional role class, usually from `AGENT_ROLE_CLASS` or `ROLE_CLASS`.
    pub role_class: Option<String>,
}

impl SchedulerAgentIdentity {
    /// Creates an identity with explicit values.
    #[must_use]
    pub fn new(agent_id: Option<String>, role_class: Option<String>) -> Self {
        Self {
            agent_id: normalize_non_empty(agent_id),
            role_class: normalize_non_empty(role_class).map(|value| value.to_ascii_lowercase()),
        }
    }

    /// Loads identity from environment variables.
    ///
    /// Resolution:
    /// - agent id: `AGENT_ID`
    /// - role class: `AGENT_ROLE_CLASS`, fallback `ROLE_CLASS`
    #[must_use]
    pub fn from_env() -> Self {
        let agent_id = std::env::var("AGENT_ID").ok();
        let role_class = std::env::var("AGENT_ROLE_CLASS")
            .ok()
            .or_else(|| std::env::var("ROLE_CLASS").ok());
        Self::new(agent_id, role_class)
    }

    /// Returns `true` when at least one identity axis is configured.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.agent_id.is_some() || self.role_class.is_some()
    }
}

fn normalize_non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|inner| inner.trim().to_string())
        .filter(|inner| !inner.is_empty())
}
