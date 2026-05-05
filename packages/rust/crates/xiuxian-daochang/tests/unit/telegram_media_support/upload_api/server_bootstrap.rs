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
    clippy::too_many_lines,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::assigning_clones
)]

use anyhow::Result;
use axum::{Router, routing::post};

use super::bootstrap::spawn_test_server;
use super::{
    MockTelegramUploadState, media_group::handle_upload_media_group, photo::handle_upload_photo,
};

pub(super) async fn spawn_upload_photo_server(
    state: MockTelegramUploadState,
) -> Result<Option<(String, MockTelegramUploadState, tokio::task::JoinHandle<()>)>> {
    let app = Router::new()
        .route("/botfake-token/sendPhoto", post(handle_upload_photo))
        .with_state(state.clone());
    spawn_test_server(
        app,
        state,
        "skipping telegram upload tests: local socket bind is not permitted",
    )
    .await
}

pub(super) async fn spawn_upload_media_group_server(
    state: MockTelegramUploadState,
) -> Result<Option<(String, MockTelegramUploadState, tokio::task::JoinHandle<()>)>> {
    let app = Router::new()
        .route(
            "/botfake-token/sendMediaGroup",
            post(handle_upload_media_group),
        )
        .with_state(state.clone());
    spawn_test_server(
        app,
        state,
        "skipping telegram media-group upload tests: local socket bind is not permitted",
    )
    .await
}
