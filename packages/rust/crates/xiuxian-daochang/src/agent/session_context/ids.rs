//! Session context backup identifier helpers.

const SESSION_CONTEXT_BACKUP_PREFIX: &str = "__session_context_backup__:";
const SESSION_CONTEXT_BACKUP_META_PREFIX: &str = "__session_context_backup_meta__:";

pub(crate) fn backup_session_id(session_id: &str) -> String {
    format!("{SESSION_CONTEXT_BACKUP_PREFIX}{session_id}")
}

pub(crate) fn backup_metadata_session_id(session_id: &str) -> String {
    format!("{SESSION_CONTEXT_BACKUP_META_PREFIX}{session_id}")
}
