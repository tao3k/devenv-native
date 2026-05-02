//! Telegram job observability branch for JSON and text summaries.

pub(crate) mod json_summary;
pub(crate) mod preview;
mod render;
mod send;

pub(in crate::channels::telegram::runtime::jobs) use preview::log_preview;
pub(in crate::channels::telegram::runtime::jobs) use send::send_with_observability;
