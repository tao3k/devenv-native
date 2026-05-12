//! `parsers::zhixing::tasks::types` owns Wendao zhixing tasks types behavior.

/// Structured zhixing agenda-task projection parsed from one markdown line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLineProjection {
    /// Human-readable task title stripped from inline metadata comments.
    pub title: String,
    /// One-based source line number in the agenda document.
    pub line_no: usize,
    /// Whether the checklist marker is completed.
    pub is_completed: bool,
    /// Optional stable task identifier from inline metadata.
    pub task_id: Option<String>,
    /// Optional priority token from inline metadata.
    pub priority: Option<String>,
    /// Carryover count resolved from metadata or inline fallback markers.
    pub carryover: u32,
    /// Optional scheduled timestamp token from inline metadata.
    pub scheduled_at: Option<String>,
    /// Optional reminded state parsed from inline metadata.
    pub reminded: Option<bool>,
}
