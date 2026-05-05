//! Telegram dispatch branch for foreground, background, and interrupts.

mod interrupt;
mod preview;
mod startup;
mod turn;
mod worker_pool;

pub(crate) use interrupt::ForegroundInterruptController;
pub(in crate::channels::telegram::runtime) use startup::start_telegram_runtime;
