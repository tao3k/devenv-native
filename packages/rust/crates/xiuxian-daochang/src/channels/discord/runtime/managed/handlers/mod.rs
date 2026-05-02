//! Discord managed handler branch for auth and command routing.

mod auth;
mod background_completion;
mod command_dispatch;
mod events;
mod send;

pub(in super::super) use background_completion::push_background_completion;
pub(crate) use command_dispatch::handle_inbound_managed_command;
