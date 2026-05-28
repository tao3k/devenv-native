//! Wendao model-routing constants.

/// Wendao model routing mode environment variable.
pub const WENDAO_MODEL_ROUTING_MODE_ENV: &str = "WENDAO_MODEL_ROUTING_MODE";
/// vLLM-SR base URL environment variable.
pub const WENDAO_VLLM_SR_BASE_URL_ENV: &str = "WENDAO_VLLM_SR_BASE_URL";
/// vLLM-SR config path environment variable.
pub const WENDAO_VLLM_SR_CONFIG_PATH_ENV: &str = "WENDAO_VLLM_SR_CONFIG_PATH";

/// Stable route id metadata header.
pub const WENDAO_ROUTE_ID_HEADER: &str = "x-wendao-route-id";
/// Stable route task-kind metadata header.
pub const WENDAO_ROUTE_TASK_KIND_HEADER: &str = "x-wendao-route-task-kind";
/// Stable route modality metadata header.
pub const WENDAO_ROUTE_MODALITY_HEADER: &str = "x-wendao-route-modality";
/// Stable selected-provider metadata header.
pub const WENDAO_ROUTE_SELECTED_PROVIDER_HEADER: &str = "x-wendao-route-selected-provider";
/// Stable selected-model metadata header.
pub const WENDAO_ROUTE_SELECTED_MODEL_HEADER: &str = "x-wendao-route-selected-model";
/// Stable selected backend-profile metadata header.
pub const WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER: &str =
    "x-wendao-route-selected-backend-profile";
/// Stable precision-tier metadata header.
pub const WENDAO_ROUTE_PRECISION_TIER_HEADER: &str = "x-wendao-route-precision-tier";

/// Default vLLM-SR proxy endpoint used by Wendao.
pub const DEFAULT_WENDAO_VLLM_SR_BASE_URL: &str = "http://127.0.0.1:8888";
/// Default local model-routing mode for developer experience.
pub const DEFAULT_WENDAO_MODEL_ROUTING_MODE: &str = "deterministic";
/// vLLM-SR auto model token used by the OpenAI-compatible data plane.
pub const VLLM_SR_AUTO_MODEL: &str = "auto";
/// vLLM-SR selected decision response header.
pub const VLLM_SR_SELECTED_DECISION_HEADER: &str = "x-vsr-selected-decision";
/// vLLM-SR selected model response header.
pub const VLLM_SR_SELECTED_MODEL_HEADER: &str = "x-vsr-selected-model";
/// vLLM-SR selected confidence response header.
pub const VLLM_SR_SELECTED_CONFIDENCE_HEADER: &str = "x-vsr-selected-confidence";
/// vLLM-SR selected reasoning response header.
pub const VLLM_SR_SELECTED_REASONING_HEADER: &str = "x-vsr-selected-reasoning";
/// vLLM-SR selected modality response header.
pub const VLLM_SR_SELECTED_MODALITY_HEADER: &str = "x-vsr-selected-modality";
/// vLLM-SR request id response header.
pub const VLLM_SR_REQUEST_ID_HEADER: &str = "x-request-id";
