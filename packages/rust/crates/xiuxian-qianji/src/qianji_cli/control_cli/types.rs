use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlCliCommand {
    RecoverySnapshot {
        ledger_path: PathBuf,
        run_id: String,
        now_ms: u64,
        json: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControlCliOutput {
    pub(crate) rendered: String,
}
