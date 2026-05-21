use xiuxian_qianji_control::{RunRecoverySnapshot, RunView};

pub(crate) struct ControlStateQueryView<'a> {
    pub(crate) event_count: usize,
    pub(crate) run_view: &'a RunView,
    pub(crate) recovery_snapshot: &'a RunRecoverySnapshot,
}
