//! Telegram command router branch for background, managed, and session routes.

mod background;
mod dispatch;
mod foreground;
mod preempt;
mod session;

pub(in crate::channels::telegram::runtime::jobs) use dispatch::handle_inbound_message_with_interrupt;
