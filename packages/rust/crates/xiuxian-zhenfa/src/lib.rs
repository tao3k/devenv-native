//! Xiuxian-Zhenfa (Matrix Gateway): contract and streaming gateway with optional JSON-RPC HTTP support.

#[cfg(feature = "client")]
mod client;
mod contracts;
#[cfg(feature = "gateway")]
mod gateway;
mod native;
mod router;
mod transmuter;
mod types;
mod xml_lite;
#[cfg(feature = "xml-transform")]
mod xml_transform;

pub use async_trait;
#[cfg(feature = "client")]
pub use client::{ZhenfaClient, ZhenfaClientError, ZhenfaClientSuccess};
pub use contracts::{
    INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JSONRPC_VERSION,
    JsonRpcErrorObject, JsonRpcId, JsonRpcMeta, JsonRpcRequest, JsonRpcResponse,
    METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE,
};
#[cfg(feature = "contract-validation")]
pub use contracts::{
    ZhenfaContractError, resolve_contract_path, validate_contract, validate_contract_reference,
};
#[cfg(feature = "gateway")]
pub use gateway::{
    HealthResponse, NotificationError, NotificationPayload, NotificationService, WebhookConfig,
    ZhenfaGatewayBuildError, ZhenfaGatewayBuilder, notification_worker,
};
pub use native::{
    BroadcastResult, ExternalSignal, ObservationSignalInput, SignalRegistry, SignalRegistryExt,
    ZhenfaContext, ZhenfaError, ZhenfaSignal,
};
#[cfg(feature = "gateway")]
pub use router::ZhenfaRouter;
pub use router::{MethodRegistry, ZhenfaMethodHandler, method_handler};
pub use schemars;
pub use serde_json;
pub use transmuter::{ZhenfaResolveAndWashError, ZhenfaTransmuter, ZhenfaTransmuterError};
pub use types::{ZhenfaSessionId, ZhenfaSignalType, ZhenfaTraceId, ZhenfaXmlLiteTagName};
pub use xml_lite::{extract_tag_f32, extract_tag_value};
#[cfg(feature = "xml-transform")]
pub use xml_transform::{json_str_to_xml, json_to_xml, markdown_to_xml};

// Re-export streaming types for xiuxian-qianji.
pub use transmuter::streaming::{
    ClaudeStreamingParser, CodexStreamingParser, CognitiveDistribution, GeminiStreamingParser,
    PipelineError, PipelineOutput, StreamProvider, StreamingOutcome, StreamingTransmuter,
    TokenUsage, ZhenfaPipeline, ZhenfaPipelineOptions, ZhenfaStreamingEvent,
};
