//! Discord partition reply branch for text and JSON rendering.

mod json;
mod text;

pub(in super::super) use json::{
    format_session_partition_admin_required_json, format_session_partition_error_json,
    format_session_partition_status_json, format_session_partition_updated_json,
};
pub(in super::super) use text::{
    format_session_partition_admin_required, format_session_partition_status,
    format_session_partition_updated,
};
