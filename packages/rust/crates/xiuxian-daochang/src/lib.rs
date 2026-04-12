//! Rust agent: one-turn loop with LLM + external tools; HTTP gateway.
//!
//! - **B.1**: Session store (in-memory or omni-window), LLM client (OpenAI-compatible chat API).
//! - **B.2**: One turn: user message → prompt + tools/list → LLM → `tool_calls` → external tool call → repeat until done.

/// Compile-time embedded resource tree rooted at `omni-agent/resources`.
pub static RESOURCES: ::include_dir::Dir<'_> =
    ::include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

mod agent;
mod channel_runtime;
mod channels;
mod config;
mod contracts;
mod embedding;
mod env_parse;
mod gateway;
mod jobs;
mod llm;
mod observability;
mod runtime_agent_factory;
mod session;
mod shortcuts;
#[doc(hidden)]
pub mod test_support;
mod tool_runtime;
mod tools;
#[doc(hidden)]
pub mod warmup_options;

pub use agent::{
    Agent, AgentStreamEvent, MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot,
    NativeToolRegistry,
    NotificationDispatcher, NotificationProvider, SessionContextBudgetClassSnapshot,
    SessionContextBudgetSnapshot, SessionContextMode, SessionContextSnapshotInfo,
    SessionContextStats, SessionContextWindowInfo, SessionMemoryRecallDecision,
    SessionMemoryRecallSnapshot,
    native_tools::registry::{NativeTool, NativeToolCallContext},
    native_tools::spider::SpiderCrawlTool,
    native_tools::wendao_search::{WendaoSearchTool, WendaoSearchToolConfig},
    native_tools::zhixing::{AgendaViewTool, JournalRecordTool, TaskAddTool},
    prune_messages_for_token_budget, summarise_drained_turns,
};
pub use channel_runtime::{
    ChannelProvider, DiscordRuntimeMode, TelegramChannelMode, WebhookDedupBackendMode,
};
pub use channels::{
    Channel, ChannelAttachment, ChannelMessage, DEFAULT_REDIS_KEY_PREFIX,
    DISCORD_MAX_MESSAGE_LENGTH, DiscordAclOverrides, DiscordChannel, DiscordCommandAdminRule,
    DiscordControlCommandPolicy, DiscordIngressApp, DiscordIngressBuildRequest,
    DiscordIngressRunRequest, DiscordRuntimeConfig, DiscordSessionPartition,
    DiscordSlashCommandPolicy, ForegroundQueueMode, RecipientCommandAdminUsersMutation,
    RecipientMentionPolicyStatus, SessionGate, TELEGRAM_MAX_MESSAGE_LENGTH, TelegramAclOverrides,
    TelegramChannel, TelegramCommandAdminRule, TelegramControlCommandPolicy, TelegramRuntimeConfig,
    TelegramSessionPartition, TelegramSlashCommandPolicy, TelegramWebhookApp,
    TelegramWebhookControlPolicyBuildRequest, TelegramWebhookPartitionBuildRequest,
    TelegramWebhookPolicyRunRequest, TelegramWebhookRunRequest, WebhookDedupBackend,
    WebhookDedupConfig, build_discord_acl_overrides, build_discord_command_admin_rule,
    build_discord_ingress_app, build_discord_ingress_app_with_control_command_policy,
    build_discord_ingress_app_with_partition_and_control_command_policy,
    build_telegram_acl_overrides, build_telegram_acl_overrides_from_settings,
    build_telegram_command_admin_rule, build_telegram_webhook_app,
    build_telegram_webhook_app_with_control_command_policy,
    build_telegram_webhook_app_with_partition, chunk_marker_reserve_chars,
    decorate_chunk_for_telegram, markdown_to_telegram_html, markdown_to_telegram_markdown_v2,
    run_discord_gateway, run_discord_ingress, run_telegram, run_telegram_webhook,
    run_telegram_webhook_with_control_command_policy, run_telegram_with_control_command_policy,
    split_message_for_discord, split_message_for_telegram,
};
pub use config::{
    AgentConfig, ContextBudgetStrategy, DiscordChannelSettings, DiscordSettings, EmbeddingSettings,
    InferenceSettings, LITELLM_DEFAULT_URL, MemoryConfig, MemorySettings, RuntimeSettings,
    SessionSettings, TelegramAclAllowSettings, TelegramAclControlSettings,
    TelegramAclPrincipalSettings, TelegramAclSettings, TelegramAclSlashSettings, TelegramSettings,
    ToolConfigFile, ToolRuntimeSettings, ToolServerEntry, ToolServerEntryFile, WendaoGatewayConfig,
    XiuxianConfig, load_runtime_settings, load_runtime_settings_from_paths, load_tool_config,
    load_xiuxian_config_from_bases, load_xiuxian_config_from_paths, set_config_home_override,
};
pub use contracts::{
    DiscoverConfidence, DiscoverMatch, GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind,
    GraphWorkflowMode, MemoryGateDecision, MemoryGateVerdict, MemoryPromotionTarget, OmegaDecision,
    OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass, RouteTrace,
    RouteTraceGraphStep, RouteTraceInjection, WorkflowBridgeMode,
};
pub use embedding::EmbeddingClient;
pub use gateway::{
    DEFAULT_STDIO_SESSION_ID, GatewayExternalToolHealthResponse, GatewayHealthResponse,
    GatewayState, MessageRequest, MessageResponse, router, run_http, run_stdio,
    validate_message_request,
};
pub use jobs::{
    HeartbeatProbeState, JobCompletion, JobCompletionKind, JobHealthState, JobManager,
    JobManagerConfig, JobMetricsSnapshot, JobState, JobStatusSnapshot, RecurringScheduleConfig,
    RecurringScheduleOutcome, TurnRunner, classify_heartbeat_probe_result, classify_job_health,
    run_recurring_schedule,
};
pub use observability::session_event_ids;
pub use runtime_agent_factory::build_agent;
pub use session::{
    BoundedSessionStore, ChatMessage, FunctionCall, SessionStore, SessionSummarySegment,
    ToolCallOut,
};
pub use shortcuts::parse_react_shortcut;
pub use tool_runtime::{
    ToolClientPool, ToolDiscoverCacheStatsSnapshot, ToolListCacheStatsSnapshot,
    ToolPoolConnectConfig, ToolRuntimeCallResult, ToolRuntimeListRequestParams,
    ToolRuntimeListResult, ToolRuntimeToolDefinition, connect_tool_pool,
};
pub use tools::{parse_qualified_tool_name, qualify_tool_name};
