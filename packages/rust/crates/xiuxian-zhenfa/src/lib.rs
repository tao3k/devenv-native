//! Xiuxian-Zhenfa (Matrix Gateway): native-first tool microkernel with an optional JSON-RPC HTTP gateway.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

mod client;
mod contracts;
mod gateway;
mod native;
mod router;
mod transmuter;
mod xml_lite;
mod xml_transform;

pub use async_trait;
pub use schemars;
pub use serde_json;
pub use xiuxian_macros::zhenfa_tool;

pub use client::{ZhenfaClient, ZhenfaClientError, ZhenfaClientSuccess};
pub use contracts::{
    INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JSONRPC_VERSION,
    JsonRpcErrorObject, JsonRpcId, JsonRpcMeta, JsonRpcRequest, JsonRpcResponse,
    METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE, ZhenfaContractError, resolve_contract_path,
    validate_contract, validate_contract_reference,
};
pub use gateway::{
    HealthResponse, NotificationError, NotificationPayload, NotificationService, WebhookConfig,
    ZhenfaGatewayBuildError, ZhenfaGatewayBuilder, notification_worker,
};
pub use native::{
    BroadcastResult, ExternalSignal, SignalRegistry, SignalRegistryExt, ZhenfaAuditSink,
    ZhenfaContext, ZhenfaDispatchEvent, ZhenfaDispatchOutcome, ZhenfaError, ZhenfaMutationGuard,
    ZhenfaMutationLock, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaRegistry,
    ZhenfaResultCache, ZhenfaSignal, ZhenfaSignalSink, ZhenfaTool,
};
pub use router::{MethodRegistry, ZhenfaMethodHandler, ZhenfaRouter, method_handler};
pub use transmuter::{ZhenfaResolveAndWashError, ZhenfaTransmuter, ZhenfaTransmuterError};
pub use xml_lite::{extract_tag_f32, extract_tag_value};
pub use xml_transform::{json_str_to_xml, json_to_xml, markdown_to_xml};

// Re-export streaming types for xiuxian-qianji
pub use transmuter::streaming::{
    ClaudeStreamingParser, CodexStreamingParser, CognitiveDistribution, GeminiStreamingParser,
    PipelineError, PipelineOutput, StreamProvider, StreamingOutcome, StreamingTransmuter,
    TokenUsage, ZhenfaPipeline, ZhenfaStreamingEvent,
};
