//! Discord managed runtime branch for handlers, replies, and scheduling.

mod handlers;
mod parsing;
mod replies;

pub(super) use handlers::{handle_inbound_managed_command, push_background_completion};
