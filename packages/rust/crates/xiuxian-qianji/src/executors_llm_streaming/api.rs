//! Api surface for `xiuxian-qianji`.

use std::sync::Arc;

use xiuxian_llm::llm::LlmClient;
use xiuxian_zhenfa::{StreamProvider, ZhenfaPipeline, ZhenfaPipelineOptions};

/// Streaming LLM analyzer with cognitive sovereignty protection.
///
/// This mechanism wraps the LLM call with `ZhenfaPipeline` to provide:
/// - Real-time cognitive state monitoring
/// - Early-halt detection for low coherence
/// - XSD validation for structured outputs
/// - Cognitive distribution metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputFlags {
    /// Whether to parse model output as JSON and store structured value.
    pub parse_json_output: bool,
    /// Whether to build a fallback shard plan from `repo_tree` when JSON parsing fails.
    pub fallback_repo_tree_on_parse_failure: bool,
}

/// Configuration for cognitive supervision during streaming analysis.
#[derive(Debug, Clone, Copy)]
pub struct PipelineFlags {
    /// Whether to enable XSD validation on output.
    pub validate_xsd: bool,
    /// Whether to enable cognitive monitoring.
    pub monitor_cognitive: bool,
}

impl Default for PipelineFlags {
    fn default() -> Self {
        Self {
            validate_xsd: true,
            monitor_cognitive: true,
        }
    }
}

/// Pipeline settings used when supervising streaming output.
#[derive(Debug, Clone, Copy)]
pub struct StreamingPipelineSettings {
    /// Early-halt threshold for cognitive coherence (0.0 to disable).
    pub early_halt_threshold: f32,
    /// Provider for streaming pipeline (default: Claude).
    pub stream_provider: StreamProvider,
    /// Boolean pipeline flags grouped to avoid a boolean-heavy analyzer surface.
    pub flags: PipelineFlags,
}

impl Default for StreamingPipelineSettings {
    fn default() -> Self {
        Self {
            early_halt_threshold: 0.0,
            stream_provider: StreamProvider::Claude,
            flags: PipelineFlags::default(),
        }
    }
}

/// Streaming analyzer that supervises LLM output with a `ZhenfaPipeline`.
pub struct StreamingLlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
    /// Context keys to extract and format into the prompt.
    pub context_keys: Vec<String>,
    /// The template/base prompt for the system.
    pub prompt_template: String,
    /// The output key to store the result.
    pub output_key: String,
    /// Flags that control how output text is interpreted.
    pub output_flags: OutputFlags,
    /// Cognitive supervision settings for the streaming pipeline.
    pub pipeline_settings: StreamingPipelineSettings,
}

impl StreamingLlmAnalyzer {
    /// Create a new streaming analyzer with default options.
    #[must_use]
    pub fn new(client: Arc<dyn LlmClient>, model: String) -> Self {
        Self {
            client,
            model,
            context_keys: Vec::new(),
            prompt_template: String::new(),
            output_key: "analysis_conclusion".to_string(),
            output_flags: OutputFlags::default(),
            pipeline_settings: StreamingPipelineSettings {
                early_halt_threshold: 0.3,
                ..StreamingPipelineSettings::default()
            },
        }
    }

    /// Create a builder for custom configuration.
    #[must_use]
    pub fn builder() -> StreamingLlmAnalyzerBuilder {
        StreamingLlmAnalyzerBuilder::default()
    }

    /// Create the `ZhenfaPipeline` for this analyzer.
    pub(crate) fn create_pipeline(&self) -> ZhenfaPipeline {
        ZhenfaPipeline::with_options(
            ZhenfaPipelineOptions::new(self.pipeline_settings.stream_provider)
                .with_xsd_validation(self.pipeline_settings.flags.validate_xsd)
                .with_cognitive_monitoring(self.pipeline_settings.flags.monitor_cognitive)
                .with_early_halt_threshold(self.pipeline_settings.early_halt_threshold),
        )
    }
}

/// Builder for `StreamingLlmAnalyzer`.
#[derive(Default)]
pub struct StreamingLlmAnalyzerBuilder {
    client: Option<Arc<dyn LlmClient>>,
    model: Option<String>,
    context_keys: Vec<String>,
    prompt_template: String,
    output_key: String,
    output_flags: OutputFlags,
    pipeline_settings: StreamingPipelineSettings,
}

impl StreamingLlmAnalyzerBuilder {
    /// Set the LLM client.
    #[must_use]
    pub fn client(mut self, client: Arc<dyn LlmClient>) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the model name.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set context keys for prompt interpolation.
    #[must_use]
    pub fn context_keys(mut self, keys: Vec<String>) -> Self {
        self.context_keys = keys;
        self
    }

    /// Set the prompt template.
    #[must_use]
    pub fn prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }

    /// Set the output key.
    #[must_use]
    pub fn output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }

    /// Enable JSON output parsing.
    #[must_use]
    pub fn parse_json_output(mut self, enabled: bool) -> Self {
        self.output_flags.parse_json_output = enabled;
        self
    }

    /// Enable fallback repo tree on parse failure.
    #[must_use]
    pub fn fallback_repo_tree(mut self, enabled: bool) -> Self {
        self.output_flags.fallback_repo_tree_on_parse_failure = enabled;
        self
    }

    /// Set early-halt threshold (0.0 to disable).
    #[must_use]
    pub fn early_halt_threshold(mut self, threshold: f32) -> Self {
        self.pipeline_settings.early_halt_threshold = threshold;
        self
    }

    /// Set the stream provider.
    #[must_use]
    pub fn stream_provider(mut self, provider: StreamProvider) -> Self {
        self.pipeline_settings.stream_provider = provider;
        self
    }

    /// Enable/disable XSD validation.
    #[must_use]
    pub fn validate_xsd(mut self, enabled: bool) -> Self {
        self.pipeline_settings.flags.validate_xsd = enabled;
        self
    }

    /// Enable/disable cognitive monitoring.
    #[must_use]
    pub fn monitor_cognitive(mut self, enabled: bool) -> Self {
        self.pipeline_settings.flags.monitor_cognitive = enabled;
        self
    }

    /// Build the analyzer.
    ///
    /// # Panics
    ///
    /// Panics if `client` or `model` is not set.
    #[must_use]
    pub fn build(self) -> StreamingLlmAnalyzer {
        let Some(client) = self.client else {
            panic!("client is required");
        };
        let Some(model) = self.model else {
            panic!("model is required");
        };
        StreamingLlmAnalyzer {
            client,
            model,
            context_keys: self.context_keys,
            prompt_template: self.prompt_template,
            output_key: self.output_key,
            output_flags: self.output_flags,
            pipeline_settings: self.pipeline_settings,
        }
    }
}
