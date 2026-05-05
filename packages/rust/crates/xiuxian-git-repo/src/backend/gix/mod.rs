//! Gix-backed repository operations for clone, fetch, checkout, and probing.

mod checkout;
mod clone;
mod constants;
mod error;
mod fetch;
mod interrupt;
mod open;
mod probe;
mod remote;
mod retry;
mod tuning;
mod types;

#[cfg(test)]
#[path = "../../../tests/unit/backend/gix/mod.rs"]
mod tests;

pub(crate) use checkout::checkout_detached_to_revision;
pub(crate) use clone::{clone_bare_with_retry, clone_checkout_from_mirror};
pub(crate) use error::BackendError;
pub(crate) use fetch::fetch_origin_with_retry;
pub(crate) use open::{open_bare_with_retry, open_checkout_with_retry};
pub(crate) use probe::probe_remote_target_revision_with_retry;
pub(crate) use remote::ensure_remote_url;
pub(crate) use retry::{is_retryable_remote_error_message, should_fetch};
pub(crate) use types::RepositoryHandle;
