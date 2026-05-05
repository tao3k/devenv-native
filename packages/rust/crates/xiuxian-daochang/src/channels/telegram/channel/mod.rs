//! Telegram channel transport and message formatting.

mod acl;
mod acl_overrides;
mod acl_reload;
mod admin_rules;
mod authorization;
mod chunking;
mod client;
mod constants;
mod constructor;
mod error;
mod group_policy;
mod identity;
mod inbound_media;
mod listen;
mod markdown;
mod media;
mod outbound_text;
mod parsing;
mod policy;
mod recipient_admin;
mod send_api;
mod send_attachments;
mod send_gate;
mod send_text;
mod send_types;
mod session_admin_persistence;
mod state;
mod trait_impl;

pub use acl_overrides::{
    TelegramAclOverrides, build_telegram_acl_overrides, build_telegram_acl_overrides_from_settings,
};
pub use admin_rules::TelegramCommandAdminRule;
pub use admin_rules::build_telegram_command_admin_rule;
pub use chunking::{
    chunk_marker_reserve_chars, decorate_chunk_for_telegram, split_message_for_telegram,
};
pub use constants::TELEGRAM_MAX_MESSAGE_LENGTH;
pub(in crate::channels::telegram::channel) use identity::parse_recipient_target;
#[doc(hidden)]
pub use markdown::{markdown_to_telegram_html, markdown_to_telegram_markdown_v2};
pub(in crate::channels::telegram::channel) use outbound_text::normalize_telegram_outbound_text;
pub(in crate::channels::telegram::channel) use policy::TelegramSlashCommandRule;
pub use policy::{TelegramControlCommandPolicy, TelegramSlashCommandPolicy};
pub(in crate::channels::telegram::channel) use send_types::PreparedCaption;
pub use state::TelegramChannel;
pub(in crate::channels::telegram::channel) use state::{
    TELEGRAM_ACL_RELOAD_CHECK_INTERVAL, TELEGRAM_API_BASE_ENV,
};
