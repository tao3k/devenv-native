//! Canonical unit test harness for `xiuxian-llm`.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/embedding_backend.rs"]
mod embedding_backend;
#[path = "unit/embedding_openai_compat.rs"]
mod embedding_openai_compat;
#[path = "unit/feature_default_litellm_only.rs"]
mod feature_default_litellm_only;
#[path = "unit/llm_acceleration_unit.rs"]
mod llm_acceleration_unit;
#[path = "unit/llm_anthropic_roles.rs"]
mod llm_anthropic_roles;
#[path = "unit/llm_backend.rs"]
mod llm_backend;
#[path = "unit/llm_multimodal.rs"]
mod llm_multimodal;
#[path = "unit/llm_openai_responses_payload.rs"]
mod llm_openai_responses_payload;
#[path = "unit/llm_openai_responses_stream.rs"]
mod llm_openai_responses_stream;
#[path = "unit/llm_openai_responses_transport/mod.rs"]
mod llm_openai_responses_transport;
#[path = "unit/llm_runtime_profile.rs"]
mod llm_runtime_profile;
#[path = "unit/llm_vision.rs"]
mod llm_vision;
#[path = "unit/test_embedding_runtime.rs"]
mod test_embedding_runtime;
#[path = "unit/web_spider_unit.rs"]
mod web_spider_unit;
