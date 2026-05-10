//! Conversation consolidation helpers for memory episode creation.

/// Drained conversation turn used for summary consolidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainedTurn {
    /// Role name from the drained session slot.
    pub role: String,
    /// Message content from the drained session slot.
    pub content: String,
    /// Number of tool calls associated with the drained slot.
    pub tool_calls: u32,
}

impl DrainedTurn {
    /// Build a drained turn record.
    #[must_use]
    pub fn new(role: impl Into<String>, content: impl Into<String>, tool_calls: u32) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            tool_calls,
        }
    }
}

/// Consolidated turn summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainedTurnSummary {
    /// First user message or a no-user fallback marker.
    pub intent: String,
    /// Joined assistant messages or a no-response fallback marker.
    pub experience: String,
    /// Normalized consolidation outcome.
    pub outcome: String,
}

/// Build intent (first user message), experience (assistant responses joined), outcome (completed/error).
#[doc(hidden)]
#[must_use]
pub fn summarise_drained_turns(drained: &[DrainedTurn]) -> DrainedTurnSummary {
    let intent = drained
        .iter()
        .find(|turn| turn.role == "user")
        .map_or("(no user message)", |turn| turn.content.as_str())
        .to_string();
    let experience: String = drained
        .iter()
        .filter(|turn| turn.role == "assistant")
        .map(|turn| turn.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let experience = if experience.is_empty() {
        "(no assistant response)".to_string()
    } else {
        experience
    };
    let has_error = drained.iter().any(|turn| {
        let lower = turn.content.to_lowercase();
        lower.contains("error") || lower.contains("failed") || lower.contains("exception")
    });
    let outcome = if has_error {
        "error".to_string()
    } else {
        "completed".to_string()
    };
    DrainedTurnSummary {
        intent,
        experience,
        outcome,
    }
}

pub(crate) fn build_consolidated_summary_text(
    intent: &str,
    experience: &str,
    outcome: &str,
) -> String {
    let intent = compact_single_line(intent, 180);
    let experience = compact_single_line(experience, 220);
    format!(
        "Outcome={outcome}; intent={intent}; assistant={experience}",
        outcome = outcome.trim(),
        intent = intent,
        experience = experience
    )
}

fn compact_single_line(input: &str, max_chars: usize) -> String {
    let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }
    let keep = max_chars.saturating_sub(3);
    let mut out = normalized.chars().take(keep).collect::<String>();
    out.push_str("...");
    out
}

pub(crate) fn now_unix_ms() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}
