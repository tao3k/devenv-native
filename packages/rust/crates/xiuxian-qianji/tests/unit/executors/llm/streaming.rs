use std::fmt::Write as _;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError};
use xiuxian_zhenfa::{CognitiveDistribution, StreamProvider};

use super::output::{
    build_repo_tree_fallback_plan, parse_json_from_text, resolve_model_for_request,
};
use super::{StreamingLlmAnalyzer, StreamingLlmAnalyzerBuilder};
use crate::contracts::FlowInstruction;

struct MockLlmClient;

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _request: ChatRequest) -> Result<String, LlmError> {
        Ok("test response".to_string())
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, LlmError> {
        Ok(Box::pin(stream::empty()))
    }
}

#[derive(Debug, Clone)]
struct StreamingAnalysisResult {
    data: serde_json::Value,
    coherence_score: f32,
    cognitive_distribution: CognitiveDistribution,
    early_halt_triggered: bool,
    instruction: FlowInstruction,
}

#[test]
fn builder_creates_analyzer() {
    let analyzer = StreamingLlmAnalyzer::builder()
        .client(Arc::new(MockLlmClient))
        .model("claude-3-opus")
        .prompt_template("You are an expert auditor.")
        .output_key("audit_result")
        .parse_json_output(true)
        .early_halt_threshold(0.4)
        .validate_xsd(true)
        .monitor_cognitive(true)
        .build();

    assert_eq!(analyzer.model, "claude-3-opus");
    assert_eq!(analyzer.prompt_template, "You are an expert auditor.");
    assert_eq!(analyzer.output_key, "audit_result");
    assert!(analyzer.output_flags.parse_json_output);
    assert!((analyzer.pipeline_settings.early_halt_threshold - 0.4).abs() < 0.001);
    assert!(analyzer.pipeline_settings.flags.validate_xsd);
    assert!(analyzer.pipeline_settings.flags.monitor_cognitive);
}

#[test]
fn default_stream_provider_is_claude() {
    assert_eq!(
        super::api::StreamingPipelineSettings::default().stream_provider,
        StreamProvider::Claude
    );
}

#[test]
fn parse_json_extracts_valid_json() {
    let json = r#"{"key": "value"}"#;
    let result = parse_json_from_text(json)
        .unwrap_or_else(|| panic!("valid JSON should parse successfully"));
    assert_eq!(result["key"], "value");
}

#[test]
fn parse_json_handles_fenced_json() {
    let fenced = r#"```json
{"nested": {"data": 123}}
```"#;
    let result = parse_json_from_text(fenced);
    assert!(result.is_some());
}

#[test]
fn parse_json_handles_json_array() {
    let array = r"[1, 2, 3]";
    let value = parse_json_from_text(array)
        .unwrap_or_else(|| panic!("valid JSON array should parse successfully"));
    let arr = value
        .as_array()
        .unwrap_or_else(|| panic!("parsed JSON value should be an array"));
    assert_eq!(arr.len(), 3);
}

#[test]
fn cognitive_distribution_defaults() {
    let dist = CognitiveDistribution::default();
    assert_eq!(dist.total(), 0);
    assert!((dist.balance() - 0.5).abs() < 0.01);
    assert!((dist.uncertainty_ratio() - 0.0).abs() < 0.01);
}

#[test]
fn parse_json_handles_empty_input() {
    let result = parse_json_from_text("");
    assert!(result.is_none());

    let result = parse_json_from_text("   ");
    assert!(result.is_none());
}

#[test]
fn parse_json_handles_malformed_input() {
    let result = parse_json_from_text("not json at all");
    assert!(result.is_none());

    let result = parse_json_from_text("{broken json");
    assert!(result.is_none());
}

#[test]
fn parse_json_handles_nested_objects() {
    let nested = r#"{"outer": {"inner": {"deep": 42}}}"#;
    let value = parse_json_from_text(nested)
        .unwrap_or_else(|| panic!("nested JSON should parse successfully"));
    assert_eq!(value["outer"]["inner"]["deep"], 42);
}

#[test]
fn parse_json_handles_mixed_array() {
    let mixed = r#"[1, "two", {"three": 3}, null]"#;
    let binding = parse_json_from_text(mixed)
        .unwrap_or_else(|| panic!("mixed JSON array should parse successfully"));
    let arr = binding
        .as_array()
        .unwrap_or_else(|| panic!("parsed JSON value should be an array"));
    assert_eq!(arr.len(), 4);
}

#[test]
fn build_repo_tree_fallback_creates_plan() {
    let context = json!({
        "repo_tree": "./src\n./docs\n./tests\n./src/deep/file.rs"
    });

    let plan = build_repo_tree_fallback_plan(&context);
    assert!(plan.is_array());
    let arr = plan
        .as_array()
        .unwrap_or_else(|| panic!("fallback plan should be a JSON array"));
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["shard_id"], "repository-overview");
}

#[test]
fn build_repo_tree_fallback_limits_paths() {
    let mut tree = String::new();
    for i in 0..20 {
        writeln!(tree, "./dir{i}")
            .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
    }
    let context = json!({ "repo_tree": tree });

    let plan = build_repo_tree_fallback_plan(&context);
    let paths = plan[0]["paths"]
        .as_array()
        .unwrap_or_else(|| panic!("fallback plan should include a paths array"));
    assert!(paths.len() <= 12);
}

#[test]
fn build_repo_tree_fallback_handles_empty_tree() {
    let context = json!({ "repo_tree": "" });
    let plan = build_repo_tree_fallback_plan(&context);
    let paths = plan[0]["paths"]
        .as_array()
        .unwrap_or_else(|| panic!("fallback plan should include a paths array"));
    assert_eq!(paths[0], ".");
}

#[test]
fn resolve_model_for_request_uses_explicit_override() {
    let context = json!({ "llm_model": "custom-model" });
    let result = resolve_model_for_request(&context, "default-model");
    assert_eq!(result, "custom-model");
}

#[test]
fn resolve_model_for_request_uses_default() {
    let context = json!({});
    let result = resolve_model_for_request(&context, "default-model");
    assert_eq!(result, "default-model");
}

#[test]
fn resolve_model_for_request_uses_fallback() {
    let context = json!({ "llm_model_fallback": "fallback-model" });
    let result = resolve_model_for_request(&context, "");
    assert_eq!(result, "fallback-model");
}

#[test]
fn streaming_analysis_result_debug_impl() {
    let result = StreamingAnalysisResult {
        data: json!({"test": "value"}),
        coherence_score: 0.85,
        cognitive_distribution: CognitiveDistribution::default(),
        early_halt_triggered: false,
        instruction: FlowInstruction::Continue,
    };

    let debug_str = format!("{result:?}");
    assert!(debug_str.contains("StreamingAnalysisResult"));
    assert!(debug_str.contains("coherence_score"));
    assert_eq!(result.data["test"], "value");
    assert!((result.coherence_score - 0.85).abs() < f32::EPSILON);
    assert_eq!(result.cognitive_distribution.total(), 0);
    assert!(!result.early_halt_triggered);
    assert_eq!(result.instruction, FlowInstruction::Continue);
}

#[test]
fn builder_allows_method_chaining() {
    let analyzer = StreamingLlmAnalyzerBuilder::default()
        .client(Arc::new(MockLlmClient))
        .model("test-model")
        .prompt_template("template")
        .output_key("result")
        .context_keys(vec!["key1".to_string(), "key2".to_string()])
        .parse_json_output(true)
        .fallback_repo_tree(true)
        .early_halt_threshold(0.5)
        .stream_provider(StreamProvider::Gemini)
        .validate_xsd(false)
        .monitor_cognitive(false)
        .build();

    assert_eq!(analyzer.model, "test-model");
    assert_eq!(analyzer.prompt_template, "template");
    assert_eq!(analyzer.output_key, "result");
    assert_eq!(analyzer.context_keys.len(), 2);
    assert!(analyzer.output_flags.parse_json_output);
    assert!(analyzer.output_flags.fallback_repo_tree_on_parse_failure);
    assert!((analyzer.pipeline_settings.early_halt_threshold - 0.5).abs() < 0.001);
    assert_eq!(
        analyzer.pipeline_settings.stream_provider,
        StreamProvider::Gemini
    );
    assert!(!analyzer.pipeline_settings.flags.validate_xsd);
    assert!(!analyzer.pipeline_settings.flags.monitor_cognitive);
}

#[test]
fn builder_default_values() {
    let analyzer = StreamingLlmAnalyzerBuilder::default()
        .client(Arc::new(MockLlmClient))
        .model("default-model")
        .build();

    assert_eq!(analyzer.model, "default-model");
    assert!(analyzer.context_keys.is_empty());
    assert!(analyzer.prompt_template.is_empty());
    assert!(analyzer.output_key.is_empty());
    assert!(!analyzer.output_flags.parse_json_output);
    assert!(!analyzer.output_flags.fallback_repo_tree_on_parse_failure);
    assert!((analyzer.pipeline_settings.early_halt_threshold - 0.0).abs() < 0.001);
    assert_eq!(
        analyzer.pipeline_settings.stream_provider,
        StreamProvider::Claude
    );
    assert!(analyzer.pipeline_settings.flags.validate_xsd);
    assert!(analyzer.pipeline_settings.flags.monitor_cognitive);
}
