//! Telegram runtime job branch for command routing and completion replies.

mod api;
mod background_completion;
mod command_handlers;
mod command_router;
pub(crate) mod observability;
mod replies;

pub(in crate::channels::telegram::runtime) use api::{
    handle_inbound_message_with_interrupt, log_preview, push_background_completion,
};
