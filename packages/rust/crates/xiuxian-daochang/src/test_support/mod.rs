//! Test-only compatibility exports for integration tests.
//!
//! This module provides stable wrappers so tests can validate parser behavior
//! without path-compiling source files via `#[path = ...]`.

mod admission;
mod bootstrap;
mod discord_runtime;
mod embedding;
mod gateway_http;
mod llm;
mod managed_parser;
mod managed_runtime;
mod memory_credit;
mod memory_feedback;
mod memory_recall;
mod memory_recall_metrics;
mod memory_recall_state;
mod memory_stream_consumer;
mod reflection;
mod result;
mod runtime_agent_factory;
mod session_context;
mod session_redis;
mod telegram_parser;
mod telegram_runtime;
mod tool_startup;
mod types;
mod warmup;
mod zhenfa;

pub use admission::{
    DownstreamAdmissionDecision, DownstreamAdmissionMetrics, DownstreamAdmissionMetricsSnapshot,
    DownstreamAdmissionPolicy, DownstreamAdmissionRejectReason, DownstreamAdmissionRuntimeSnapshot,
    DownstreamInFlightSnapshot, DownstreamRuntimeSnapshot,
};
pub use bootstrap::{
    init_persona_registries_internal_len, load_skill_templates_from_embedded_registry,
    resolve_notebook_root, resolve_prj_data_home, resolve_project_root, resolve_template_globs,
};
pub use discord_runtime::{
    DiscordForegroundInterruptController, DiscordForegroundRuntimeHarness,
    build_discord_foreground_runtime, discord_interrupted_reply_is_suppressed,
    process_discord_message, process_discord_message_with_interrupt,
    push_discord_background_completion, resolve_discord_snapshot_interval_secs,
};
pub use embedding::{EmbeddingBackendMode, embed_http, parse_embedding_client_backend_mode};
#[cfg(feature = "agent-provider-litellm")]
pub use embedding::{
    NormalizedLitellmEmbeddingTarget, OLLAMA_PLACEHOLDER_API_KEY,
    normalize_litellm_embedding_target, normalize_openai_compatible_base_url,
};
pub use gateway_http::{
    GatewayEmbeddingRuntimeHandle, apply_gateway_embedding_memory_guard_for_tests,
    build_embedding_runtime_for_settings, fallback_hash_embed_batch, read_api_key,
    resolve_embed_base_url, resolve_embed_model, resolve_request_model,
    resolve_runtime_embed_base_url, resolve_target_api_key_env, resolve_target_base_url,
};
pub use llm::{
    ChatCompletionRequest, DEFAULT_ANTHROPIC_KEY_ENV, DEFAULT_MINIMAX_KEY_ENV,
    DEFAULT_OPENAI_KEY_ENV, LiteLlmProviderMode, LiteLlmWireApi, LlmBackendMode, ProviderSettings,
    ToolMessageIntegrityReport, chat_completion_request_to_value, enforce_tool_message_integrity,
    extract_api_base_from_inference_url, is_openai_like_stream_required_error, parse_backend_mode,
    parse_tools_json, resolve_backend_mode_for_inference_url, resolve_provider_settings_with_env,
    should_use_openai_like_for_base,
};
#[cfg(feature = "agent-provider-litellm")]
pub use llm::{
    CustomBaseFallbackTransport, build_responses_payload_from_chat_completion_request,
    chat_message_to_litellm_message, chat_message_to_litellm_message_for_openai_chat,
    parse_responses_stream_tool_names, resolve_custom_base_transport_api_key_from_values,
};
pub use managed_parser::{detect_managed_control_command, detect_managed_slash_command};
pub use managed_runtime::{
    SessionPartitionPersistenceTarget, build_session_id, classify_turn_error,
    persist_session_partition_mode_if_enabled, persist_session_partition_mode_to_path,
    resolve_session_partition_persist_enabled,
};
pub use memory_credit::{
    RecallCreditUpdate, RecallOutcome, RecalledEpisodeCandidate, apply_recall_credit,
    sanitize_decay_factor, select_recall_credit_candidates, should_apply_decay,
};
pub use memory_feedback::{
    FeedbackOutcome, RECALL_FEEDBACK_SOURCE_ASSISTANT, RECALL_FEEDBACK_SOURCE_COMMAND,
    RECALL_FEEDBACK_SOURCE_TOOL, RECALL_FEEDBACK_SOURCE_USER, ToolExecutionSummary,
    apply_feedback_to_plan, classify_assistant_outcome, parse_explicit_user_feedback,
    resolve_feedback_outcome, update_feedback_bias,
};
pub use memory_recall::{
    MemoryRecallInput, MemoryRecallPlan, build_memory_context_message, filter_recalled_episodes,
    filter_recalled_episodes_at, plan_memory_recall,
};
pub use memory_recall_metrics::{TestMemoryRecallMetricsState, ratio_as_f32};
pub use memory_recall_state::{
    EMBEDDING_SOURCE_EMBEDDING, EMBEDDING_SOURCE_EMBEDDING_REPAIRED, EMBEDDING_SOURCE_UNKNOWN,
    SessionMemoryRecallDecision, SessionMemoryRecallSnapshot,
    append_memory_recall_snapshot_payload, record_memory_recall_snapshot, snapshot_session_id,
};
pub use memory_stream_consumer::{
    MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, StreamReadErrorKind,
    ack_and_record_metrics, build_consumer_name, classify_stream_read_error,
    compute_retry_backoff_ms, ensure_consumer_group, is_idle_poll_timeout_error,
    parse_xreadgroup_reply, queue_promoted_candidate, read_stream_events,
    should_surface_repeated_failure, stream_consumer_connection_config,
    stream_consumer_response_timeout, summarize_redis_error,
};
pub use reflection::{
    TestPolicyHintDirective, TestReflectiveRuntime, TestReflectiveRuntimeError,
    TestReflectiveRuntimeStage, TestTurnReflection, test_build_turn_reflection,
    test_derive_policy_hint,
};
pub use result::TestSupportResult;
pub use runtime_agent_factory::{
    RuntimeMemoryResolution, parse_embedding_backend_mode, resolve_inference_url,
    resolve_runtime_embedding_backend_mode, resolve_runtime_embedding_base_url,
    resolve_runtime_inference_url, resolve_runtime_memory_options, resolve_runtime_model,
    validate_inference_url_origin,
};
pub use session_context::{
    bounded_recent_messages, bounded_recent_summary_segments, build_session_context_test_agent,
    enforce_session_reset_policy, now_unix_ms, session_messages, set_session_last_activity,
    set_session_reset_idle_timeout_ms,
};
pub use session_redis::{
    EncodedChatMessagePayload, decode_chat_message_payload, encode_chat_message_payload,
};
pub use telegram_parser::{
    test_is_agenda_command, test_is_reset_context_command, test_is_stop_command,
    test_parse_background_prompt, test_parse_help_command, test_parse_job_status_command,
    test_parse_jobs_summary_command, test_parse_resume_context_command,
    test_parse_session_admin_command, test_parse_session_context_budget_command,
    test_parse_session_context_memory_command, test_parse_session_context_status_command,
    test_parse_session_feedback_command, test_parse_session_injection_command,
    test_parse_session_mention_command, test_parse_session_partition_command,
};
pub use telegram_runtime::{
    TelegramForegroundInterruptController, handle_telegram_inbound_message_with_interrupt,
    push_telegram_background_completion, resolve_telegram_snapshot_interval_secs,
    telegram_log_preview,
};
pub use tool_startup::startup_connect_config;
pub use types::{
    JobStatusCommand, ManagedControlCommand, ManagedSlashCommand, OutputFormat,
    ResumeContextCommand, SessionAdminAction, SessionAdminCommand, SessionFeedbackCommand,
    SessionFeedbackDirection, SessionInjectionAction, SessionInjectionCommand,
    SessionMentionCommand, SessionMentionMode, SessionPartitionCommand, SessionPartitionMode,
};
pub use warmup::{WarmupEnvOverrides, WarmupOptions, resolve_warmup_options};
pub use zhenfa::{
    ZhenfaRuntimeDeps, ZhenfaToolBridge, ZhenfaValkeyHookConfig, build_zhenfa_orchestrator_hooks,
    memory_reward_signal_sink, memory_reward_signal_sink_with_valkey_backend,
    resolve_zhenfa_valkey_hook_config,
};
