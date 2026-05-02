//! Valkey-backed Telegram session-gate branch for locks and cleanup.

mod acquire;
mod commands;
mod guard;
mod state;
mod token;

pub(super) use state::{
    DEFAULT_GATE_RETRY_INTERVAL_MS, DistributedLeaseGuard, NEXT_LEASE_OWNER_ID,
    ValkeySessionGateBackend,
};
