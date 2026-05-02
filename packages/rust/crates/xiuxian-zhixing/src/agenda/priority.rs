//! Agenda priority levels for cultivation tasks.

use serde::{Deserialize, Serialize};

/// Priority levels for cultivation tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    /// Routine tasks (Daily maintenance).
    Low,
    /// Standard progress (Core curriculum).
    #[default]
    Medium,
    /// Critical breakthroughs or deadlines (Tribulation/Manifestation).
    High,
}
