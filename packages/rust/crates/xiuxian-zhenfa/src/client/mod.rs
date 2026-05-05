//! HTTP JSON-RPC client facade for Zhenfa gateway calls.

mod api;

pub use api::{ZhenfaClient, ZhenfaClientError, ZhenfaClientSuccess};
