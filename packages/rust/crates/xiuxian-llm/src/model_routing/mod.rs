//! Model routing contracts shared by Wendao Gateway and model-plane adapters.

mod attachment;
mod chat;
mod client;
mod config;
mod constants;
mod metadata;
mod route_helpers;
mod types;

pub use attachment::{
    DEFAULT_WENDAO_ATTACHMENT_ROUTE_PROVIDER,
    DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE,
    DEFAULT_WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL,
    DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE, DEFAULT_WENDAO_IMAGE_EXTRACT_ROUTE_MODEL,
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_AUDIO_TRANSCRIPT_ROUTE_MODEL_ENV,
    WENDAO_AUDIO_TRANSCRIPT_ROUTE_PROVIDER_ENV, WENDAO_IMAGE_EXTRACT_ROUTE_BACKEND_PROFILE_ENV,
    WENDAO_IMAGE_EXTRACT_ROUTE_MODEL_ENV, WENDAO_IMAGE_EXTRACT_ROUTE_PROVIDER_ENV,
    WendaoAttachmentRouteConfig, WendaoAttachmentRouteInput,
    wendao_attachment_model_route_decision, wendao_attachment_route_intent,
    wendao_audio_transcript_route_config_with_lookup,
    wendao_audio_transcript_route_config_with_model_routing_config,
    wendao_image_extract_route_config_with_lookup,
    wendao_image_extract_route_config_with_model_routing_config,
};
pub use chat::{
    DEFAULT_WENDAO_CHAT_ROUTE_BACKEND_PROFILE, DEFAULT_WENDAO_CHAT_ROUTE_MODEL,
    WENDAO_CHAT_ROUTE_BACKEND_PROFILE_ENV, WENDAO_CHAT_ROUTE_MODEL_ENV,
    WENDAO_CHAT_ROUTE_PROVIDER_ENV, WendaoChatRouteConfig, WendaoChatRouteInput,
    wendao_chat_model_route_decision, wendao_chat_route_config_with_lookup,
    wendao_chat_route_config_with_model_routing_config, wendao_chat_route_intent,
};
pub use client::VllmSrRouteDecisionClient;
pub use config::{
    WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML, WendaoModelRoutingTomlConfig, WendaoRouteTomlConfig,
    wendao_model_routing_config_from_toml_str, wendao_model_routing_config_from_toml_value,
    wendao_model_routing_system_default_config,
};
pub use constants::{
    DEFAULT_WENDAO_MODEL_ROUTING_MODE, DEFAULT_WENDAO_VLLM_SR_BASE_URL, VLLM_SR_AUTO_MODEL,
    VLLM_SR_REQUEST_ID_HEADER, VLLM_SR_SELECTED_CONFIDENCE_HEADER,
    VLLM_SR_SELECTED_DECISION_HEADER, VLLM_SR_SELECTED_MODALITY_HEADER,
    VLLM_SR_SELECTED_MODEL_HEADER, VLLM_SR_SELECTED_REASONING_HEADER,
    WENDAO_MODEL_ROUTING_MODE_ENV, WENDAO_ROUTE_ID_HEADER, WENDAO_ROUTE_MODALITY_HEADER,
    WENDAO_ROUTE_PRECISION_TIER_HEADER, WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
    WENDAO_ROUTE_SELECTED_MODEL_HEADER, WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
    WENDAO_ROUTE_TASK_KIND_HEADER, WENDAO_VLLM_SR_BASE_URL_ENV, WENDAO_VLLM_SR_CONFIG_PATH_ENV,
};
pub use metadata::wendao_model_route_metadata;
pub use types::{
    WendaoModelDecision, WendaoModelRoutingMode, WendaoRouteIntent, WendaoRouteSourceKind,
    WendaoRouteTaskKind, wendao_model_routing_mode_with_lookup,
    wendao_vllm_sr_base_url_with_lookup,
};
