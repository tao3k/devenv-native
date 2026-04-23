//! Waiting-state runtime records and bounded event-poll helpers.

mod competition;
mod poll;

pub(crate) use poll::{apply_event_poll_outcome_impl, build_event_poll_request_impl};
