use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlCliCommand {
    History {
        ledger_path: PathBuf,
        run_id: String,
        json: bool,
    },
    RecoverySnapshot {
        ledger_path: PathBuf,
        run_id: String,
        now_ms: u64,
        json: bool,
    },
    View {
        ledger_path: PathBuf,
        run_id: String,
        json: bool,
    },
    Step {
        ledger_path: PathBuf,
        run_id: String,
        step_id: String,
        json: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControlCliOutput {
    pub(crate) rendered: String,
}
