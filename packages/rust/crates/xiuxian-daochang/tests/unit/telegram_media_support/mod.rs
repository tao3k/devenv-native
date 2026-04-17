//! Test coverage for xiuxian-daochang behavior.

mod bootstrap;
pub(crate) mod media_api;
pub(crate) mod upload_api;

pub(crate) use media_api::{
    MediaCall, spawn_mock_telegram_media_api, spawn_mock_telegram_media_api_with_group_failure,
    spawn_mock_telegram_media_api_with_markdown_error,
};
pub(crate) use upload_api::{
    spawn_mock_telegram_media_group_upload_api, spawn_mock_telegram_upload_api,
    spawn_mock_telegram_upload_api_with_markdown_error,
};
