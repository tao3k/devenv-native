use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlCliCommand {
    Activity {
        ledger_path: PathBuf,
        run_id: String,
        step_id: Option<String>,
        activity_id: String,
        json: bool,
    },
    Decision {
        ledger_path: PathBuf,
        run_id: String,
        step_id: Option<String>,
        decision_id: String,
        json: bool,
    },
    History {
        ledger_path: PathBuf,
        run_id: String,
        json: bool,
    },
    Heartbeat {
        ledger_path: PathBuf,
        run_id: String,
        worker_id: String,
        observed_at_ms: u64,
        expires_at_ms: u64,
        metadata: Option<String>,
        json: bool,
    },
    QueryState {
        ledger_path: PathBuf,
        run_id: String,
        now_ms: u64,
        json: bool,
    },
    RecoverySnapshot {
        ledger_path: PathBuf,
        run_id: String,
        now_ms: u64,
        json: bool,
    },
    Signal {
        ledger_path: PathBuf,
        run_id: String,
        step_id: Option<String>,
        signal_name: String,
        payload: String,
        received_at_ms: u64,
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
    Timer {
        ledger_path: PathBuf,
        run_id: String,
        step_id: Option<String>,
        timer_id: String,
        json: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControlCliOutput {
    pub(crate) rendered: String,
}
