#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

#[path = "media_api/markdown_fallback.rs"]
mod markdown_fallback;
#[path = "media_api/routing.rs"]
mod routing;
#[path = "media_api/server_bootstrap.rs"]
mod server_bootstrap;
use super::bootstrap;
use server_bootstrap::spawn_media_api_server;

#[derive(Clone, Debug)]
pub struct MediaCall {
    pub method: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Default)]
pub struct MockTelegramMediaState {
    pub calls: Arc<Mutex<Vec<MediaCall>>>,
    pub fail_send_media_group_remaining: Arc<Mutex<usize>>,
    pub first_markdown_error: Arc<Mutex<Option<String>>>,
}

impl MockTelegramMediaState {
    fn with_failures_and_markdown_error(
        fail_send_media_group_times: usize,
        first_markdown_error: Option<&str>,
    ) -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            fail_send_media_group_remaining: Arc::new(Mutex::new(fail_send_media_group_times)),
            first_markdown_error: Arc::new(Mutex::new(
                first_markdown_error.map(std::string::ToString::to_string),
            )),
        }
    }
}

pub async fn spawn_mock_telegram_media_api_with_group_failure_and_markdown_error(
    fail_send_media_group_times: usize,
    first_markdown_error: Option<&str>,
) -> Result<Option<(String, MockTelegramMediaState, tokio::task::JoinHandle<()>)>> {
    let state = MockTelegramMediaState::with_failures_and_markdown_error(
        fail_send_media_group_times,
        first_markdown_error,
    );
    spawn_media_api_server(state).await
}

pub async fn spawn_mock_telegram_media_api()
-> Result<Option<(String, MockTelegramMediaState, tokio::task::JoinHandle<()>)>> {
    spawn_mock_telegram_media_api_with_group_failure_and_markdown_error(0, None).await
}

pub async fn spawn_mock_telegram_media_api_with_group_failure(
    fail_send_media_group_times: usize,
) -> Result<Option<(String, MockTelegramMediaState, tokio::task::JoinHandle<()>)>> {
    spawn_mock_telegram_media_api_with_group_failure_and_markdown_error(
        fail_send_media_group_times,
        None,
    )
    .await
}

pub async fn spawn_mock_telegram_media_api_with_markdown_error(
    first_markdown_error: Option<&str>,
) -> Result<Option<(String, MockTelegramMediaState, tokio::task::JoinHandle<()>)>> {
    spawn_mock_telegram_media_api_with_group_failure_and_markdown_error(0, first_markdown_error)
        .await
}
