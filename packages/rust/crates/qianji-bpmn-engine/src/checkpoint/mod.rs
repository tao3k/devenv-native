//! Checkpoint api seam over internal persistence helpers.

mod api;
mod codec;
mod keys;
#[cfg(feature = "valkey")]
mod lease;
#[cfg(feature = "sqlite")]
mod sql;
#[cfg(feature = "valkey")]
mod valkey;

pub(crate) use api::{decode_checkpoint_json_impl, encode_checkpoint_json_impl};
#[cfg(feature = "valkey")]
pub(crate) use api::{
    delete_checkpoint_as_owner_impl, delete_checkpoint_impl, load_checkpoint_impl,
    save_checkpoint_as_owner_impl, save_checkpoint_impl,
};
#[cfg(feature = "sqlite")]
pub(crate) use api::{
    delete_checkpoint_sql_impl, load_checkpoint_sql_impl, save_checkpoint_sql_impl,
};
pub(crate) use api::{lease_key_impl, state_key_impl};
#[cfg(feature = "valkey")]
pub(crate) use api::{
    release_checkpoint_lease_impl, renew_checkpoint_lease_impl, try_acquire_checkpoint_lease_impl,
};
