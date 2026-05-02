use crate::channels::discord::runtime::ForegroundInterruptController;

pub(super) fn begin_active_generation(
    interrupt_controller: &ForegroundInterruptController,
    session_id: &str,
) -> (
    tokio::sync::watch::Receiver<u64>,
    ActiveGenerationGuard,
    u64,
) {
    let interrupt_rx = interrupt_controller.begin_generation(session_id);
    let active_generation_guard =
        ActiveGenerationGuard::new(interrupt_controller.clone(), session_id.to_string());
    let interrupt_generation = *interrupt_rx.borrow();
    (interrupt_rx, active_generation_guard, interrupt_generation)
}

#[derive(Clone)]
pub(super) struct ActiveGenerationGuard {
    controller: ForegroundInterruptController,
    session_id: String,
}

impl ActiveGenerationGuard {
    fn new(controller: ForegroundInterruptController, session_id: String) -> Self {
        Self {
            controller,
            session_id,
        }
    }
}

impl Drop for ActiveGenerationGuard {
    fn drop(&mut self) {
        self.controller.end_generation(&self.session_id);
    }
}
