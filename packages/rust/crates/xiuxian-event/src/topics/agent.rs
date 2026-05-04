//! Agent lifecycle topic grouping.

use super::{AGENT_ACTION, AGENT_RESULT, AGENT_THINK};

/// Agent lifecycle topics.
pub const AGENT_TOPICS: &[(&str, &str)] = &[
    ("THINK", AGENT_THINK),
    ("ACTION", AGENT_ACTION),
    ("RESULT", AGENT_RESULT),
];
