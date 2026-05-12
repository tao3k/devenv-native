//! Domain-owned live Flight host for SearchStrategyFlow replay.
//!
//! The transport crate stays pure: this module owns the Wendao graph/search
//! semantics needed by a real replay host.

mod config;
mod providers;
mod repo_content;
mod server;

pub use server::{FlightHostResult, run_repo_search_flight_server_from_args};
