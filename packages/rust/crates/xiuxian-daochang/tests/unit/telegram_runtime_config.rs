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
use std::collections::HashMap;

use xiuxian_daochang::{TelegramRuntimeConfig, TelegramSettings};

#[test]
fn defaults_are_applied_when_env_missing() {
    let cfg = TelegramRuntimeConfig::from_lookup_for_test(|_| None, None);
    assert_eq!(cfg.inbound_queue_capacity, 100);
    assert_eq!(cfg.foreground_queue_capacity, 256);
    assert_eq!(cfg.foreground_max_in_flight_messages, 16);
    assert_eq!(cfg.foreground_turn_timeout_secs, 80);
}

#[test]
fn valid_env_values_override_defaults() {
    let values = HashMap::from([
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_INBOUND_QUEUE_CAPACITY",
            "200".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_QUEUE_CAPACITY",
            "512".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_MAX_IN_FLIGHT",
            "32".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_TURN_TIMEOUT_SECS",
            "600".to_string(),
        ),
    ]);
    let cfg = TelegramRuntimeConfig::from_lookup_for_test(|name| values.get(name).cloned(), None);
    assert_eq!(cfg.inbound_queue_capacity, 200);
    assert_eq!(cfg.foreground_queue_capacity, 512);
    assert_eq!(cfg.foreground_max_in_flight_messages, 32);
    assert_eq!(cfg.foreground_turn_timeout_secs, 600);
}

#[test]
fn invalid_values_fall_back_to_defaults() {
    let values = HashMap::from([
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_INBOUND_QUEUE_CAPACITY",
            "0".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_QUEUE_CAPACITY",
            "-3".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_MAX_IN_FLIGHT",
            "not-a-number".to_string(),
        ),
        (
            "XIUXIAN_DAOCHANG_TELEGRAM_FOREGROUND_TURN_TIMEOUT_SECS",
            "0".to_string(),
        ),
    ]);
    let cfg = TelegramRuntimeConfig::from_lookup_for_test(|name| values.get(name).cloned(), None);
    assert_eq!(cfg.inbound_queue_capacity, 100);
    assert_eq!(cfg.foreground_queue_capacity, 256);
    assert_eq!(cfg.foreground_max_in_flight_messages, 16);
    assert_eq!(cfg.foreground_turn_timeout_secs, 80);
}

#[test]
fn settings_values_used_when_env_missing() {
    let settings = TelegramSettings {
        inbound_queue_capacity: Some(123),
        foreground_queue_capacity: Some(456),
        foreground_max_in_flight_messages: Some(7),
        foreground_turn_timeout_secs: Some(42),
        ..Default::default()
    };
    let cfg = TelegramRuntimeConfig::from_lookup_for_test(|_| None, Some(&settings));
    assert_eq!(cfg.inbound_queue_capacity, 123);
    assert_eq!(cfg.foreground_queue_capacity, 456);
    assert_eq!(cfg.foreground_max_in_flight_messages, 7);
    assert_eq!(cfg.foreground_turn_timeout_secs, 42);
}
