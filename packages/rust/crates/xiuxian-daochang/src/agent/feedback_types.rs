//! Public memory recall feedback result types.

/// Explicit session-level recall feedback direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRecallFeedbackDirection {
    Up,
    Down,
}

/// Result of applying explicit session-level recall feedback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionRecallFeedbackUpdate {
    pub previous_bias: f32,
    pub updated_bias: f32,
    pub direction: SessionRecallFeedbackDirection,
}
