//! Owns the Studio document extract audio shard Flight client surface.

use std::path::PathBuf;
use std::sync::Arc;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::{TryStreamExt, stream};
use tonic::transport::{Channel, Endpoint};
use xiuxian_qianji::workflow_kernel::WorkflowMemoryCheckpointRecord;
use xiuxian_qianji::{
    WorkflowMemoryCheckpointStore, WorkflowRun, WorkflowStage, WorkflowStageFacts,
    WorkflowTopology, WorkflowTrace,
};
use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardMaterializationInput, AudioShardMaterializedItem,
    AudioShardMergeReport, AudioShardPlan, AudioShardResult, AudioShardWorkerProfile,
    AudioSpeechWindowPlannerInput, AudioTranscriptAdmissionOptions, build_audio_shard_input_batch,
    build_audio_shard_inputs, build_audio_speech_window_plan, decode_audio_shard_result_batches,
    materialize_audio_shards, merge_audio_shard_results,
};
use xiuxian_wendao_server::transport::{
    ANALYSIS_AUDIO_SHARDS_ROUTE, WENDAO_AUDIO_HOSTED_BASE_URL_HEADER,
    WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER, WENDAO_AUDIO_HOSTED_MODEL_HEADER,
    WENDAO_AUDIO_HOSTED_PROVIDER_HEADER, WENDAO_AUDIO_WORKER_HEADER, WENDAO_AUDIO_WORKERS_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
};

const AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;
const AUDIO_SHARD_WORKFLOW_ID: &str = "wendao.audio_shards.flight.v1";
const AUDIO_MATERIALIZE_STAGE_ID: &str = "audio.materialize_shards";
const AUDIO_BUILD_ROWS_STAGE_ID: &str = "audio.build_arrow_rows";
const AUDIO_CALL_FLIGHT_STAGE_ID: &str = "audio.call_analyzer_flight";
const AUDIO_MERGE_GATE_STAGE_ID: &str = "audio.merge_precision_gate";
const AUDIO_INPUT_BATCH_CHECKPOINT_ID: &str = "audio.arrow.input_batch.v1";
const AUDIO_RESULT_BATCHES_CHECKPOINT_ID: &str = "audio.arrow.result_batches.v1";

/// Feature-gated Arrow Flight client for the internal audio shard exchange.
#[derive(Debug, Clone)]
pub struct AudioShardFlightClient {
    endpoint_url: String,
    channel: Channel,
}

/// Audio shard worker response decoded into typed rows.
#[derive(Debug, Clone)]
pub struct AudioShardFlightResponse {
    /// Typed audio result rows returned by the Python analyzer worker.
    pub results: Vec<AudioShardResult>,
}

/// Optional request metadata for one analyzer audio shard exchange.
#[derive(Debug, Clone, Default)]
pub struct AudioShardFlightRequestOptions {
    /// Optional Python worker budget.
    pub worker_budget: Option<usize>,
    /// Optional analyzer audio worker selector.
    pub audio_worker: Option<String>,
    /// Optional hosted audio provider override.
    pub hosted_provider: Option<String>,
    /// Optional hosted audio base URL override.
    pub hosted_base_url: Option<String>,
    /// Optional hosted audio endpoint-kind override.
    pub hosted_endpoint: Option<String>,
    /// Optional hosted audio model override.
    pub hosted_model: Option<String>,
    /// Optional Rust-owned transcript transcript admission root.
    pub transcript_admission_dir: Option<PathBuf>,
}

impl AudioShardFlightRequestOptions {
    pub(crate) fn with_worker_budget(&self, worker_budget: Option<usize>) -> Self {
        Self {
            worker_budget,
            audio_worker: self.audio_worker.clone(),
            hosted_provider: self.hosted_provider.clone(),
            hosted_base_url: self.hosted_base_url.clone(),
            hosted_endpoint: self.hosted_endpoint.clone(),
            hosted_model: self.hosted_model.clone(),
            transcript_admission_dir: self.transcript_admission_dir.clone(),
        }
    }

    pub(crate) fn transcript_admission_options(&self) -> AudioTranscriptAdmissionOptions {
        AudioTranscriptAdmissionOptions {
            audio_worker: self.audio_worker.clone(),
            hosted_provider: self.hosted_provider.clone(),
            hosted_base_url: self.hosted_base_url.clone(),
            hosted_endpoint: self.hosted_endpoint.clone(),
            hosted_model: self.hosted_model.clone(),
            admission_dir: self.transcript_admission_dir.clone(),
        }
    }
}

/// Typed execution report for the Studio audio shard workflow proof.
#[derive(Debug, Clone)]
pub struct AudioShardWorkflowExecution {
    /// Rust-owned shard plan used for this execution.
    pub plan: AudioShardPlan,
    /// Rust-materialized shard artifacts.
    pub materialized_shards: Vec<AudioShardMaterializedItem>,
    /// Arrow-contract input rows sent to the analyzer Flight route.
    pub inputs: Vec<AudioShardInput>,
    /// Analyzer Flight response rows.
    pub response: AudioShardFlightResponse,
    /// Merge report produced from the returned rows and submitted input rows.
    pub merge_report: AudioShardMergeReport,
    /// Qianji workflow-kernel trace for the typed execution chain.
    pub trace: WorkflowTrace,
    /// Same-process memory checkpoints retained by the workflow run.
    pub memory_checkpoints: WorkflowMemoryCheckpointStore,
}

impl AudioShardFlightResponse {
    /// Merge the response rows against the submitted shard inputs.
    ///
    /// # Errors
    ///
    /// Returns an error when result rows fail identity, fingerprint, profile,
    /// or text MIME validation.
    pub fn merge_for_inputs(
        &self,
        inputs: &[AudioShardInput],
    ) -> Result<AudioShardMergeReport, String> {
        merge_audio_shard_results(inputs, self.results.as_slice())
    }
}

impl AudioShardFlightClient {
    /// Connect to the Python analyzer Flight endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint URL is invalid or cannot be reached.
    pub async fn connect(endpoint_url: impl Into<String>) -> Result<Self, String> {
        let endpoint_url = endpoint_url.into();
        let endpoint = Endpoint::from_shared(endpoint_url.clone())
            .map_err(|error| format!("invalid audio shard endpoint `{endpoint_url}`: {error}"))?;
        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect audio shard endpoint `{endpoint_url}`: {error}")
        })?;
        Ok(Self {
            endpoint_url,
            channel,
        })
    }

    /// Return the connected endpoint URL.
    #[must_use]
    pub fn endpoint_url(&self) -> &str {
        self.endpoint_url.as_str()
    }

    /// Send audio shard input rows and decode worker result rows.
    ///
    /// # Errors
    ///
    /// Returns an error when input rows are empty, Arrow encoding fails, the
    /// Flight exchange fails, or the worker response does not match the stable
    /// audio shard result contract.
    pub async fn request(
        &self,
        inputs: &[AudioShardInput],
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_with_worker_budget(inputs, None).await
    }

    /// Build audio shard input rows from Rust-materialized shards and send them
    /// to the analyzer worker.
    ///
    /// # Errors
    ///
    /// Returns an error when the materialized shard list is empty, Arrow
    /// encoding fails, the Flight exchange fails, or the worker response does
    /// not match the stable audio shard result contract.
    pub async fn request_materialized(
        &self,
        shards: &[AudioShardMaterializedItem],
        profile: &AudioShardWorkerProfile,
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_materialized_with_worker_budget(shards, profile, None)
            .await
    }

    /// Materialize a Rust-owned audio shard plan and send it to the analyzer
    /// worker.
    ///
    /// # Errors
    ///
    /// Returns an error when planning or materialization fails, the Flight
    /// exchange fails, or the worker response does not match the stable audio
    /// shard result contract.
    pub async fn request_plan(
        &self,
        plan: &AudioShardPlan,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_plan_with_worker_budget(plan, materialization, profile, None)
            .await
    }

    /// Build a speech-window plan from model-neutral timing facts, materialize
    /// it, and send it to the analyzer worker.
    ///
    /// # Errors
    ///
    /// Returns an error when speech-window planning or materialization fails,
    /// the Flight exchange fails, or the worker response does not match the
    /// stable audio shard result contract.
    pub async fn request_speech_window_plan(
        &self,
        planner_input: &AudioSpeechWindowPlannerInput,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_speech_window_plan_with_worker_budget(
            planner_input,
            materialization,
            profile,
            None,
        )
        .await
    }

    /// Send audio shard inputs with an optional Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the Flight exchange fails or the worker response
    /// does not match the stable audio shard result contract.
    pub async fn request_with_worker_budget(
        &self,
        inputs: &[AudioShardInput],
        worker_budget: Option<usize>,
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_with_options(
            inputs,
            &AudioShardFlightRequestOptions {
                worker_budget,
                ..AudioShardFlightRequestOptions::default()
            },
        )
        .await
    }

    /// Send audio shard inputs with request metadata options.
    ///
    /// # Errors
    ///
    /// Returns an error when the Flight exchange fails or the worker response
    /// does not match the stable audio shard result contract.
    pub async fn request_with_options(
        &self,
        inputs: &[AudioShardInput],
        options: &AudioShardFlightRequestOptions,
    ) -> Result<AudioShardFlightResponse, String> {
        request_audio_shards_on_channel(self.channel.clone(), inputs, options).await
    }

    /// Build audio shard input rows from Rust-materialized shards and send them
    /// with an optional Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the materialized shard list is empty, Arrow
    /// encoding fails, the Flight exchange fails, or the worker response does
    /// not match the stable audio shard result contract.
    pub async fn request_materialized_with_worker_budget(
        &self,
        shards: &[AudioShardMaterializedItem],
        profile: &AudioShardWorkerProfile,
        worker_budget: Option<usize>,
    ) -> Result<AudioShardFlightResponse, String> {
        let inputs = build_audio_shard_inputs(shards, profile);
        self.request_with_worker_budget(inputs.as_slice(), worker_budget)
            .await
    }

    /// Materialize a Rust-owned audio shard plan and send it with an optional
    /// Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when planning or materialization fails, the Flight
    /// exchange fails, or the worker response does not match the stable audio
    /// shard result contract.
    pub async fn request_plan_with_worker_budget(
        &self,
        plan: &AudioShardPlan,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
        worker_budget: Option<usize>,
    ) -> Result<AudioShardFlightResponse, String> {
        let shards = materialize_audio_shards(plan, materialization)?;
        self.request_materialized_with_worker_budget(shards.as_slice(), profile, worker_budget)
            .await
    }

    /// Build a speech-window plan from model-neutral timing facts, materialize
    /// it, and send it with an optional Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when speech-window planning or materialization fails,
    /// the Flight exchange fails, or the worker response does not match the
    /// stable audio shard result contract.
    pub async fn request_speech_window_plan_with_worker_budget(
        &self,
        planner_input: &AudioSpeechWindowPlannerInput,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
        worker_budget: Option<usize>,
    ) -> Result<AudioShardFlightResponse, String> {
        let plan = build_audio_speech_window_plan(planner_input)?;
        self.request_plan_with_worker_budget(&plan, materialization, profile, worker_budget)
            .await
    }

    /// Execute a Rust-owned audio shard plan through the Qianji typed workflow
    /// kernel proof.
    ///
    /// # Errors
    ///
    /// Returns an error when materialization, Arrow row construction, Flight
    /// exchange, or merge validation fails.
    pub async fn execute_plan_with_worker_budget(
        &self,
        plan: &AudioShardPlan,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
        worker_budget: Option<usize>,
    ) -> Result<AudioShardWorkflowExecution, String> {
        let topology = audio_shard_workflow_topology()?;
        let mut run =
            WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
        let mut context = AudioShardWorkflowContext {
            client: self.clone(),
            materialization: materialization.clone(),
            profile: profile.clone(),
            worker_budget,
            request_options: AudioShardFlightRequestOptions::default(),
        };

        let materialized_shards = run
            .run_stage(&mut context, &MaterializeAudioShardPlanStage, plan.clone())
            .await
            .map_err(|error| error.to_string())?;
        let prepared_inputs = run
            .run_stage(
                &mut context,
                &BuildAudioShardInputsStage,
                materialized_shards.clone(),
            )
            .await
            .map_err(|error| error.to_string())?;
        run.record_memory_checkpoint(WorkflowMemoryCheckpointRecord {
            stage_id: AUDIO_BUILD_ROWS_STAGE_ID.into(),
            checkpoint_id: AUDIO_INPUT_BATCH_CHECKPOINT_ID.into(),
            facts: WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_input", "v1")
                .with_item_count(prepared_inputs.input_batch.num_rows()),
            content_fingerprint: None,
            payload: Arc::new(prepared_inputs.input_batch.clone()),
        })
        .map_err(|error| error.to_string())?;
        let exchange = run
            .run_stage(
                &mut context,
                &RequestAudioShardFlightStage,
                prepared_inputs.clone(),
            )
            .await
            .map_err(|error| error.to_string())?;
        run.record_memory_checkpoint(WorkflowMemoryCheckpointRecord {
            stage_id: AUDIO_CALL_FLIGHT_STAGE_ID.into(),
            checkpoint_id: AUDIO_RESULT_BATCHES_CHECKPOINT_ID.into(),
            facts: WorkflowStageFacts::arrow_record_batch(
                "xiuxian_wendao.audio_shard_result",
                "v1",
            )
            .with_item_count(exchange.response.results.len()),
            content_fingerprint: None,
            payload: Arc::new(exchange.response_batches.clone()),
        })
        .map_err(|error| error.to_string())?;
        let merge_report = run
            .run_stage(
                &mut context,
                &MergeAudioShardResultsStage,
                AudioShardMergeStageInput {
                    inputs: prepared_inputs.inputs.clone(),
                    response: exchange.response.clone(),
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        let workflow_report = run
            .finish_checked(merge_report.clone())
            .map_err(|error| error.to_string())?;

        Ok(AudioShardWorkflowExecution {
            plan: plan.clone(),
            materialized_shards,
            inputs: prepared_inputs.inputs,
            response: exchange.response,
            merge_report: workflow_report.output,
            trace: workflow_report.trace,
            memory_checkpoints: workflow_report.memory_checkpoints,
        })
    }

    /// Build a speech-window plan and execute it through the Qianji typed
    /// workflow kernel proof.
    ///
    /// # Errors
    ///
    /// Returns an error when speech-window planning, materialization, Flight
    /// exchange, or merge validation fails.
    pub async fn execute_speech_window_plan_with_worker_budget(
        &self,
        planner_input: &AudioSpeechWindowPlannerInput,
        materialization: &AudioShardMaterializationInput,
        profile: &AudioShardWorkerProfile,
        worker_budget: Option<usize>,
    ) -> Result<AudioShardWorkflowExecution, String> {
        let plan = build_audio_speech_window_plan(planner_input)?;
        self.execute_plan_with_worker_budget(&plan, materialization, profile, worker_budget)
            .await
    }
}

#[derive(Debug, Clone)]
struct AudioShardWorkflowContext {
    client: AudioShardFlightClient,
    materialization: AudioShardMaterializationInput,
    profile: AudioShardWorkerProfile,
    worker_budget: Option<usize>,
    request_options: AudioShardFlightRequestOptions,
}

#[derive(Debug, Clone, Copy)]
struct MaterializeAudioShardPlanStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardWorkflowContext, AudioShardPlan> for MaterializeAudioShardPlanStage {
    type Output = Vec<AudioShardMaterializedItem>;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_MATERIALIZE_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardPlan) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardPlan").with_item_count(input.start_offsets_ms.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("Vec<AudioShardMaterializedItem>").with_item_count(output.len())
    }

    async fn run(
        &self,
        context: &mut AudioShardWorkflowContext,
        input: AudioShardPlan,
    ) -> Result<Self::Output, String> {
        materialize_audio_shards(&input, &context.materialization)
    }
}

#[derive(Debug, Clone, Copy)]
struct BuildAudioShardInputsStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardWorkflowContext, Vec<AudioShardMaterializedItem>>
    for BuildAudioShardInputsStage
{
    type Output = AudioShardPreparedInputs;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_BUILD_ROWS_STAGE_ID
    }

    fn input_facts(&self, input: &Vec<AudioShardMaterializedItem>) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("Vec<AudioShardMaterializedItem>").with_item_count(input.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_input", "v1")
            .with_item_count(output.inputs.len())
    }

    async fn run(
        &self,
        context: &mut AudioShardWorkflowContext,
        input: Vec<AudioShardMaterializedItem>,
    ) -> Result<Self::Output, String> {
        let inputs = build_audio_shard_inputs(input.as_slice(), &context.profile);
        let input_batch = build_audio_shard_input_batch(inputs.as_slice())?;
        Ok(AudioShardPreparedInputs {
            inputs,
            input_batch,
        })
    }
}

#[derive(Debug, Clone)]
struct AudioShardPreparedInputs {
    inputs: Vec<AudioShardInput>,
    input_batch: EngineRecordBatch,
}

#[derive(Debug, Clone, Copy)]
struct RequestAudioShardFlightStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardWorkflowContext, AudioShardPreparedInputs>
    for RequestAudioShardFlightStage
{
    type Output = AudioShardFlightExchange;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_CALL_FLIGHT_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardPreparedInputs) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_input", "v1")
            .with_item_count(input.inputs.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_result", "v1")
            .with_item_count(output.response.results.len())
    }

    async fn run(
        &self,
        context: &mut AudioShardWorkflowContext,
        input: AudioShardPreparedInputs,
    ) -> Result<Self::Output, String> {
        request_audio_shards_batch_on_channel(
            context.client.channel.clone(),
            input.input_batch,
            &context
                .request_options
                .with_worker_budget(context.worker_budget),
        )
        .await
    }
}

#[derive(Debug, Clone)]
struct AudioShardFlightExchange {
    response: AudioShardFlightResponse,
    response_batches: Vec<EngineRecordBatch>,
}

#[derive(Debug, Clone)]
struct AudioShardMergeStageInput {
    inputs: Vec<AudioShardInput>,
    response: AudioShardFlightResponse,
}

#[derive(Debug, Clone, Copy)]
struct MergeAudioShardResultsStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardWorkflowContext, AudioShardMergeStageInput>
    for MergeAudioShardResultsStage
{
    type Output = AudioShardMergeReport;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_MERGE_GATE_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardMergeStageInput) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_result", "v1")
            .with_item_count(input.response.results.len())
    }

    fn output_facts(&self, _output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardMergeReport")
    }

    async fn run(
        &self,
        _context: &mut AudioShardWorkflowContext,
        input: AudioShardMergeStageInput,
    ) -> Result<Self::Output, String> {
        input.response.merge_for_inputs(input.inputs.as_slice())
    }
}

fn audio_shard_workflow_topology() -> Result<WorkflowTopology, String> {
    WorkflowTopology::linear(
        AUDIO_SHARD_WORKFLOW_ID,
        [
            AUDIO_MATERIALIZE_STAGE_ID,
            AUDIO_BUILD_ROWS_STAGE_ID,
            AUDIO_CALL_FLIGHT_STAGE_ID,
            AUDIO_MERGE_GATE_STAGE_ID,
        ],
    )
    .map_err(|error| error.to_string())
}

async fn request_audio_shards_on_channel(
    channel: Channel,
    inputs: &[AudioShardInput],
    options: &AudioShardFlightRequestOptions,
) -> Result<AudioShardFlightResponse, String> {
    if inputs.is_empty() {
        return Err("audio shard request inputs cannot be empty".to_owned());
    }

    let input_batch = build_audio_shard_input_batch(inputs)?;
    Ok(
        request_audio_shards_batch_on_channel(channel, input_batch, options)
            .await?
            .response,
    )
}

async fn request_audio_shards_batch_on_channel(
    channel: Channel,
    input_batch: EngineRecordBatch,
    options: &AudioShardFlightRequestOptions,
) -> Result<AudioShardFlightExchange, String> {
    let request_stream = FlightDataEncoderBuilder::new()
        .with_schema(input_batch.schema())
        .with_flight_descriptor(Some(audio_shards_descriptor()))
        .with_max_flight_data_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .build(stream::iter(vec![Ok::<
            EngineRecordBatch,
            arrow_flight::error::FlightError,
        >(input_batch)]));

    let inner_client = TonicFlightServiceClient::new(channel)
        .max_encoding_message_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .max_decoding_message_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES);
    let mut client = FlightClient::new_from_inner(inner_client);
    client
        .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
        .map_err(|error| format!("invalid audio shard schema-version header: {error}"))?;
    let worker_budget_header = options
        .worker_budget
        .filter(|budget| *budget > 0)
        .map(|budget| budget.to_string());
    if let Some(worker_budget_header) = worker_budget_header.as_deref() {
        client
            .add_header(WENDAO_AUDIO_WORKERS_HEADER, worker_budget_header)
            .map_err(|error| format!("invalid audio workers header: {error}"))?;
    }
    add_optional_audio_header(
        &mut client,
        WENDAO_AUDIO_WORKER_HEADER,
        options.audio_worker.as_deref(),
        "audio worker",
    )?;
    add_optional_audio_header(
        &mut client,
        WENDAO_AUDIO_HOSTED_PROVIDER_HEADER,
        options.hosted_provider.as_deref(),
        "hosted audio provider",
    )?;
    add_optional_audio_header(
        &mut client,
        WENDAO_AUDIO_HOSTED_BASE_URL_HEADER,
        options.hosted_base_url.as_deref(),
        "hosted audio base URL",
    )?;
    add_optional_audio_header(
        &mut client,
        WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER,
        options.hosted_endpoint.as_deref(),
        "hosted audio endpoint",
    )?;
    add_optional_audio_header(
        &mut client,
        WENDAO_AUDIO_HOSTED_MODEL_HEADER,
        options.hosted_model.as_deref(),
        "hosted audio model",
    )?;
    let response_batches = client
        .do_exchange(request_stream)
        .await
        .map_err(|error| format!("audio shard exchange failed: {error}"))?
        .try_collect::<Vec<EngineRecordBatch>>()
        .await
        .map_err(|error| format!("failed to decode audio shard response: {error}"))?;
    if response_batches.is_empty() {
        return Err("audio shard exchange returned no record batches".to_owned());
    }
    let response = AudioShardFlightResponse {
        results: decode_audio_shard_result_batches(response_batches.as_slice())?,
    };

    Ok(AudioShardFlightExchange {
        response,
        response_batches,
    })
}

fn add_optional_audio_header(
    client: &mut FlightClient,
    header: &'static str,
    value: Option<&str>,
    label: &str,
) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    client
        .add_header(header, value)
        .map_err(|error| format!("invalid {label} header: {error}"))
}

fn audio_shards_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        ANALYSIS_AUDIO_SHARDS_ROUTE
            .trim_start_matches('/')
            .split('/')
            .map(ToString::to_string)
            .collect(),
    )
}
