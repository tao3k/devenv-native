//! Checkpoint model and scaffold persistence entrypoints.

mod codec;
mod keys;
#[cfg(feature = "valkey")]
mod lease;
mod model;
#[cfg(feature = "sqlite")]
mod sql;
#[cfg(feature = "valkey")]
mod valkey;

pub use codec::{decode_checkpoint_json, encode_checkpoint_json};
pub use keys::{lease_key, state_key};
#[cfg(feature = "valkey")]
pub use lease::{release_checkpoint_lease, renew_checkpoint_lease, try_acquire_checkpoint_lease};
pub use model::{BPMN_CHECKPOINT_FORMAT_VERSION, BpmnCheckpointEnvelope};
#[cfg(feature = "sqlite")]
pub use sql::{delete_checkpoint_sql, load_checkpoint_sql, save_checkpoint_sql};
#[cfg(feature = "valkey")]
pub use valkey::{
    delete_checkpoint, delete_checkpoint_as_owner, load_checkpoint, save_checkpoint,
    save_checkpoint_as_owner,
};
