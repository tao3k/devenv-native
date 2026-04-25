//! BPMN CLI parser facade; `dispatch` is the canonical visible owner.

mod dispatch;
mod resume;
mod start;
mod status;

pub(crate) use dispatch::parse_bpmn_command;
