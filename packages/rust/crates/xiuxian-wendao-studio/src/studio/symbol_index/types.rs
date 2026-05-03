use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolIndexPhase {
    Idle,
    Indexing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolIndexStatus {
    pub(crate) phase: SymbolIndexPhase,
    pub(crate) last_error: Option<String>,
    pub(crate) updated_at: Option<String>,
}

impl Default for SymbolIndexStatus {
    fn default() -> Self {
        Self {
            phase: SymbolIndexPhase::Idle,
            last_error: None,
            updated_at: Some(Utc::now().to_rfc3339()),
        }
    }
}
