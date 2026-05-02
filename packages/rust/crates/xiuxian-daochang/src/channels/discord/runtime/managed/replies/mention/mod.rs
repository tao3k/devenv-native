//! Discord mention-policy reply branch for text and JSON rendering.

mod json;
mod text;

pub(in super::super) use json::{
    format_session_mention_admin_required_json, format_session_mention_status_json,
    format_session_mention_updated_json,
};
pub(in super::super) use text::{
    format_session_mention_admin_required, format_session_mention_status,
    format_session_mention_updated,
};
