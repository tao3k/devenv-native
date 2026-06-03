//! Gateway command implementation - starts the Axum HTTP server.
//!
//! This module starts the Wendao API gateway server with:
//! - REST API endpoints for knowledge graph operations
//! - VFS access endpoints
//! - Health check endpoints
//! - Webhook notification integration
//! - Signal propagation to `NotificationService`

mod auth;
mod command;
mod config;
mod health;
mod policy;
#[cfg(feature = "postgres-auth")]
mod postgres_auth;
mod query;
mod registry;
mod security;
pub(crate) mod state;
mod status;

pub(crate) use command::handle;

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/mod.rs"]
mod tests;
