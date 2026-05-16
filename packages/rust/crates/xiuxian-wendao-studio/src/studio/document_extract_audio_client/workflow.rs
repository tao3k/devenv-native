//! Qianji workflow proof for the two-pass audio shard recovery chain.

use std::sync::Arc;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_qianji::{
    WorkflowMemoryCheckpointStore, WorkflowRun, WorkflowStage, WorkflowStageBinding,
    WorkflowStageFacts, WorkflowTopology, WorkflowTopologyEdge, WorkflowTrace,
};
use xiuxian_wendao_attachments::audio::AudioSpeechWindowPlannerInput;
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRecoveryPatchGateReport, AudioShardInput,
    AudioShardMaterializationInput, AudioShardMaterializedItem, AudioShardMergeReport,
    AudioShardPlan, AudioShardRequestMetric, AudioShardWorkerProfile,
    build_audio_shard_input_batch, build_audio_shard_inputs, build_audio_shard_result_batch,
    materialize_audio_shards,
};

use super::recovery::{
    AudioShardRecoveryPlanRequest, AudioShardRecoveryPlanning, empty_patch_gate_report,
};
use super::{AudioShardFlightClient, AudioShardFlightResponse};
use xiuxian_wendao_attachments::audio::AudioRiskParentSelectionOptions;

const AUDIO_RECOVERY_WORKFLOW_ID: &str = "wendao.audio_shards.recovery.v1";
const AUDIO_BASE_MATERIALIZE_STAGE_ID: &str = "audio.base.materialize_shards";
const AUDIO_BASE_BUILD_ROWS_STAGE_ID: &str = "audio.base.build_arrow_rows";
const AUDIO_BASE_CALL_FLIGHT_STAGE_ID: &str = "audio.base.call_analyzer_flight";
const AUDIO_PLAN_RECOVERY_STAGE_ID: &str = "audio.recovery.plan_split";
const AUDIO_RECOVERY_MATERIALIZE_STAGE_ID: &str = "audio.recovery.materialize_shards";
const AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID: &str = "audio.recovery.build_arrow_rows";
const AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID: &str = "audio.recovery.call_analyzer_flight";
const AUDIO_RECOVERY_MERGE_STAGE_ID: &str = "audio.recovery.merge_precision_gate";
const AUDIO_BASE_INPUT_BATCH_CHECKPOINT_ID: &str = "audio.base.arrow.input_batch.v1";
const AUDIO_BASE_RESULT_BATCH_CHECKPOINT_ID: &str = "audio.base.arrow.result_batch.v1";
const AUDIO_RECOVERY_INPUT_BATCH_CHECKPOINT_ID: &str = "audio.recovery.arrow.input_batch.v1";
const AUDIO_RECOVERY_RESULT_BATCH_CHECKPOINT_ID: &str = "audio.recovery.arrow.result_batch.v1";

/// Named request for executing a base audio pass plus an optional recovery pass.
#[derive(Debug, Clone, Copy)]
pub struct AudioShardRecoveryWorkflowRequest<'a> {
    /// Parent/base shard plan.
    pub parent_plan: &'a AudioShardPlan,
    /// Materialization settings used for both base and recovery shards.
    pub materialization: &'a AudioShardMaterializationInput,
    /// Analyzer worker profile used to build stable Arrow input rows.
    pub profile: &'a AudioShardWorkerProfile,
    /// Optional Rust-observed request latencies for base rows.
    pub request_metrics: &'a [AudioShardRequestMetric],
    /// Risk parent selection thresholds.
    pub selection_options: AudioRiskParentSelectionOptions,
    /// Recovery patch precision thresholds.
    pub patch_options: AudioRecoveryPatchGateOptions,
    /// Short-window split duration in milliseconds.
    pub recovery_split_duration_ms: u64,
    /// Optional model-neutral speech timing facts for recovery planning.
    pub recovery_speech_window_input: Option<&'a AudioSpeechWindowPlannerInput>,
    /// Optional worker budget for the base analyzer request.
    pub base_worker_budget: Option<usize>,
    /// Optional worker budget for the recovery analyzer request.
    pub recovery_worker_budget: Option<usize>,
}

/// Typed result for the base plus recovery audio execution path.
#[derive(Debug, Clone)]
pub struct AudioShardRecoveryWorkflowExecution {
    /// Rust-materialized base shard artifacts.
    pub base_materialized_shards: Vec<AudioShardMaterializedItem>,
    /// Base Arrow-contract input rows sent to the analyzer Flight route.
    pub base_inputs: Vec<AudioShardInput>,
    /// Base analyzer response rows.
    pub base_response: AudioShardFlightResponse,
    /// Rust-planned recovery selection and split plan.
    pub recovery_planning: AudioShardRecoveryPlanning,
    /// Rust-materialized recovery shard artifacts.
    pub recovery_materialized_shards: Vec<AudioShardMaterializedItem>,
    /// Recovery Arrow-contract input rows sent to the analyzer Flight route.
    pub recovery_inputs: Vec<AudioShardInput>,
    /// Recovery analyzer response rows, when recovery work was selected.
    pub recovery_response: Option<AudioShardFlightResponse>,
    /// Final merge report after accepted recovery patches are applied.
    pub merge_report: AudioShardMergeReport,
    /// Recovery patch precision gate report.
    pub patch_gate_report: AudioRecoveryPatchGateReport,
    /// Qianji workflow-kernel trace for the two-pass recovery chain.
    pub trace: WorkflowTrace,
    /// Same-process memory checkpoints retained by the workflow run.
    pub memory_checkpoints: WorkflowMemoryCheckpointStore,
}

impl AudioShardFlightClient {
    /// Execute a base audio analyzer pass, plan short-window recovery in Rust,
    /// execute the recovery pass when needed, and merge through the Rust patch
    /// gate.
    ///
    /// # Errors
    ///
    /// Returns an error when materialization, either Flight exchange, recovery
    /// planning, patch gating, workflow validation, or final merge validation
    /// fails.
    pub async fn execute_recovery_split(
        &self,
        request: AudioShardRecoveryWorkflowRequest<'_>,
    ) -> Result<AudioShardRecoveryWorkflowExecution, String> {
        let topology = audio_shard_recovery_workflow_topology()?;
        let mut run =
            WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
        let mut context = AudioShardRecoveryWorkflowContext {
            client: self.clone(),
            materialization: request.materialization.clone(),
            profile: request.profile.clone(),
            base_worker_budget: request.base_worker_budget,
            recovery_worker_budget: request.recovery_worker_budget,
        };

        let base_pass = run_audio_base_pass(&mut run, &mut context, request.parent_plan).await?;
        let recovery_planning =
            plan_audio_recovery(&mut run, &mut context, &request, &base_pass).await?;
        let recovery_pass =
            run_audio_recovery_pass(&mut run, &mut context, &recovery_planning).await?;
        let merge_output = merge_audio_recovery_pass(
            &mut run,
            &mut context,
            request.patch_options,
            &base_pass,
            &recovery_pass,
        )
        .await?;
        let workflow_report = run
            .finish_checked(merge_output)
            .map_err(|error| error.to_string())?;

        Ok(AudioShardRecoveryWorkflowExecution {
            base_materialized_shards: base_pass.materialized_shards,
            base_inputs: base_pass.prepared_inputs.inputs,
            base_response: base_pass.exchange.response,
            recovery_planning,
            recovery_materialized_shards: recovery_pass.materialized_shards,
            recovery_inputs: recovery_pass.inputs,
            recovery_response: recovery_pass.response,
            merge_report: workflow_report.output.merge_report,
            patch_gate_report: workflow_report.output.patch_gate_report,
            trace: workflow_report.trace,
            memory_checkpoints: workflow_report.memory_checkpoints,
        })
    }
}

#[derive(Debug, Clone)]
struct AudioShardRecoveryWorkflowContext {
    client: AudioShardFlightClient,
    materialization: AudioShardMaterializationInput,
    profile: AudioShardWorkerProfile,
    base_worker_budget: Option<usize>,
    recovery_worker_budget: Option<usize>,
}

#[derive(Debug, Clone)]
struct AudioShardPreparedInputs {
    inputs: Vec<AudioShardInput>,
    input_batch: EngineRecordBatch,
}

#[derive(Debug, Clone)]
struct AudioShardRecoveryFlightExchange {
    response: AudioShardFlightResponse,
    result_batch: EngineRecordBatch,
}

#[derive(Debug, Clone)]
struct AudioShardBaseRecoveryPass {
    materialized_shards: Vec<AudioShardMaterializedItem>,
    prepared_inputs: AudioShardPreparedInputs,
    exchange: AudioShardRecoveryFlightExchange,
}

#[derive(Debug, Clone)]
struct AudioShardShortWindowRecoveryPass {
    materialized_shards: Vec<AudioShardMaterializedItem>,
    inputs: Vec<AudioShardInput>,
    response: Option<AudioShardFlightResponse>,
}

#[derive(Debug, Clone)]
struct AudioShardRecoveryPlanningStageInput {
    parent_plan: AudioShardPlan,
    inputs: Vec<AudioShardInput>,
    response: AudioShardFlightResponse,
    request_metrics: Vec<AudioShardRequestMetric>,
    selection_options: AudioRiskParentSelectionOptions,
    split_duration_ms: u64,
    speech_window_input: Option<AudioSpeechWindowPlannerInput>,
}

#[derive(Debug, Clone)]
struct AudioShardRecoveryMergeStageInput {
    base_inputs: Vec<AudioShardInput>,
    base_response: AudioShardFlightResponse,
    recovery_inputs: Vec<AudioShardInput>,
    recovery_response: Option<AudioShardFlightResponse>,
    patch_options: AudioRecoveryPatchGateOptions,
}

#[derive(Debug, Clone)]
struct AudioShardRecoveryMergeStageOutput {
    merge_report: AudioShardMergeReport,
    patch_gate_report: AudioRecoveryPatchGateReport,
}

async fn run_audio_base_pass(
    run: &mut WorkflowRun,
    context: &mut AudioShardRecoveryWorkflowContext,
    parent_plan: &AudioShardPlan,
) -> Result<AudioShardBaseRecoveryPass, String> {
    let materialized_shards = run
        .run_stage(
            context,
            &BaseMaterializeAudioShardPlanStage,
            parent_plan.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let prepared_inputs = run
        .run_stage(
            context,
            &BaseBuildAudioShardInputsStage,
            materialized_shards.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    record_recovery_workflow_checkpoint(
        run,
        AUDIO_BASE_BUILD_ROWS_STAGE_ID,
        AUDIO_BASE_INPUT_BATCH_CHECKPOINT_ID,
        "xiuxian_wendao.audio_shard_input",
        prepared_inputs.input_batch.num_rows(),
        Arc::new(prepared_inputs.input_batch.clone()),
    )?;
    let exchange = run
        .run_stage(
            context,
            &BaseRequestAudioShardFlightStage,
            prepared_inputs.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    record_recovery_workflow_checkpoint(
        run,
        AUDIO_BASE_CALL_FLIGHT_STAGE_ID,
        AUDIO_BASE_RESULT_BATCH_CHECKPOINT_ID,
        "xiuxian_wendao.audio_shard_result",
        exchange.response.results.len(),
        Arc::new(exchange.result_batch.clone()),
    )?;
    Ok(AudioShardBaseRecoveryPass {
        materialized_shards,
        prepared_inputs,
        exchange,
    })
}

async fn plan_audio_recovery(
    run: &mut WorkflowRun,
    context: &mut AudioShardRecoveryWorkflowContext,
    request: &AudioShardRecoveryWorkflowRequest<'_>,
    base_pass: &AudioShardBaseRecoveryPass,
) -> Result<AudioShardRecoveryPlanning, String> {
    run.run_stage(
        context,
        &PlanAudioShardRecoveryStage,
        AudioShardRecoveryPlanningStageInput {
            parent_plan: request.parent_plan.clone(),
            inputs: base_pass.prepared_inputs.inputs.clone(),
            response: base_pass.exchange.response.clone(),
            request_metrics: request.request_metrics.to_vec(),
            selection_options: request.selection_options,
            split_duration_ms: request.recovery_split_duration_ms,
            speech_window_input: request.recovery_speech_window_input.cloned(),
        },
    )
    .await
    .map_err(|error| error.to_string())
}

async fn run_audio_recovery_pass(
    run: &mut WorkflowRun,
    context: &mut AudioShardRecoveryWorkflowContext,
    recovery_planning: &AudioShardRecoveryPlanning,
) -> Result<AudioShardShortWindowRecoveryPass, String> {
    if recovery_planning.selected_parent_inputs.is_empty()
        || recovery_planning.recovery_plan.start_offsets_ms.is_empty()
    {
        return Ok(AudioShardShortWindowRecoveryPass {
            materialized_shards: Vec::new(),
            inputs: Vec::new(),
            response: None,
        });
    }

    let materialized_shards = run
        .run_stage(
            context,
            &RecoveryMaterializeAudioShardPlanStage,
            recovery_planning.recovery_plan.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let prepared_inputs = run
        .run_stage(
            context,
            &RecoveryBuildAudioShardInputsStage,
            materialized_shards.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    record_recovery_workflow_checkpoint(
        run,
        AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID,
        AUDIO_RECOVERY_INPUT_BATCH_CHECKPOINT_ID,
        "xiuxian_wendao.audio_shard_input",
        prepared_inputs.input_batch.num_rows(),
        Arc::new(prepared_inputs.input_batch.clone()),
    )?;
    let exchange = run
        .run_stage(
            context,
            &RecoveryRequestAudioShardFlightStage,
            prepared_inputs.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    record_recovery_workflow_checkpoint(
        run,
        AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID,
        AUDIO_RECOVERY_RESULT_BATCH_CHECKPOINT_ID,
        "xiuxian_wendao.audio_shard_result",
        exchange.response.results.len(),
        Arc::new(exchange.result_batch.clone()),
    )?;
    Ok(AudioShardShortWindowRecoveryPass {
        materialized_shards,
        inputs: prepared_inputs.inputs,
        response: Some(exchange.response),
    })
}

async fn merge_audio_recovery_pass(
    run: &mut WorkflowRun,
    context: &mut AudioShardRecoveryWorkflowContext,
    patch_options: AudioRecoveryPatchGateOptions,
    base_pass: &AudioShardBaseRecoveryPass,
    recovery_pass: &AudioShardShortWindowRecoveryPass,
) -> Result<AudioShardRecoveryMergeStageOutput, String> {
    run.run_stage(
        context,
        &MergeAudioShardRecoveryStage,
        AudioShardRecoveryMergeStageInput {
            base_inputs: base_pass.prepared_inputs.inputs.clone(),
            base_response: base_pass.exchange.response.clone(),
            recovery_inputs: recovery_pass.inputs.clone(),
            recovery_response: recovery_pass.response.clone(),
            patch_options,
        },
    )
    .await
    .map_err(|error| error.to_string())
}

fn record_recovery_workflow_checkpoint<T>(
    run: &mut WorkflowRun,
    stage_id: &'static str,
    checkpoint_id: &'static str,
    schema_name: &'static str,
    item_count: usize,
    payload: Arc<T>,
) -> Result<(), String>
where
    T: std::any::Any + Send + Sync + 'static,
{
    run.record_memory_checkpoint(
        stage_id,
        checkpoint_id,
        WorkflowStageFacts::arrow_record_batch(schema_name, "v1").with_item_count(item_count),
        None,
        payload,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Copy)]
struct BaseMaterializeAudioShardPlanStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardPlan>
    for BaseMaterializeAudioShardPlanStage
{
    type Output = Vec<AudioShardMaterializedItem>;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_BASE_MATERIALIZE_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardPlan) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardPlan").with_item_count(input.start_offsets_ms.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("Vec<AudioShardMaterializedItem>").with_item_count(output.len())
    }

    async fn run(
        &self,
        context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardPlan,
    ) -> Result<Self::Output, String> {
        materialize_audio_shards(&input, &context.materialization)
    }
}

#[derive(Debug, Clone, Copy)]
struct RecoveryMaterializeAudioShardPlanStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardPlan>
    for RecoveryMaterializeAudioShardPlanStage
{
    type Output = Vec<AudioShardMaterializedItem>;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_RECOVERY_MATERIALIZE_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardPlan) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardPlan").with_item_count(input.start_offsets_ms.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("Vec<AudioShardMaterializedItem>").with_item_count(output.len())
    }

    async fn run(
        &self,
        context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardPlan,
    ) -> Result<Self::Output, String> {
        materialize_audio_shards(&input, &context.materialization)
    }
}

#[derive(Debug, Clone, Copy)]
struct BaseBuildAudioShardInputsStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, Vec<AudioShardMaterializedItem>>
    for BaseBuildAudioShardInputsStage
{
    type Output = AudioShardPreparedInputs;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_BASE_BUILD_ROWS_STAGE_ID
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
        context: &mut AudioShardRecoveryWorkflowContext,
        input: Vec<AudioShardMaterializedItem>,
    ) -> Result<Self::Output, String> {
        build_prepared_inputs(input, &context.profile)
    }
}

#[derive(Debug, Clone, Copy)]
struct RecoveryBuildAudioShardInputsStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, Vec<AudioShardMaterializedItem>>
    for RecoveryBuildAudioShardInputsStage
{
    type Output = AudioShardPreparedInputs;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID
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
        context: &mut AudioShardRecoveryWorkflowContext,
        input: Vec<AudioShardMaterializedItem>,
    ) -> Result<Self::Output, String> {
        build_prepared_inputs(input, &context.profile)
    }
}

#[derive(Debug, Clone, Copy)]
struct BaseRequestAudioShardFlightStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardPreparedInputs>
    for BaseRequestAudioShardFlightStage
{
    type Output = AudioShardRecoveryFlightExchange;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_BASE_CALL_FLIGHT_STAGE_ID
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
        context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardPreparedInputs,
    ) -> Result<Self::Output, String> {
        request_with_result_batch(&context.client, input.inputs, context.base_worker_budget).await
    }
}

#[derive(Debug, Clone, Copy)]
struct RecoveryRequestAudioShardFlightStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardPreparedInputs>
    for RecoveryRequestAudioShardFlightStage
{
    type Output = AudioShardRecoveryFlightExchange;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID
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
        context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardPreparedInputs,
    ) -> Result<Self::Output, String> {
        request_with_result_batch(
            &context.client,
            input.inputs,
            context.recovery_worker_budget,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
struct PlanAudioShardRecoveryStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardRecoveryPlanningStageInput>
    for PlanAudioShardRecoveryStage
{
    type Output = AudioShardRecoveryPlanning;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_PLAN_RECOVERY_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardRecoveryPlanningStageInput) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_result", "v1")
            .with_item_count(input.response.results.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardRecoveryPlanning")
            .with_item_count(output.selected_parent_inputs.len())
    }

    async fn run(
        &self,
        _context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardRecoveryPlanningStageInput,
    ) -> Result<Self::Output, String> {
        input
            .response
            .plan_recovery_split(AudioShardRecoveryPlanRequest {
                parent_plan: &input.parent_plan,
                inputs: input.inputs.as_slice(),
                request_metrics: input.request_metrics.as_slice(),
                selection_options: input.selection_options,
                split_duration_ms: input.split_duration_ms,
                speech_window_input: input.speech_window_input.as_ref(),
            })
    }
}

#[derive(Debug, Clone, Copy)]
struct MergeAudioShardRecoveryStage;

#[async_trait::async_trait]
impl WorkflowStage<AudioShardRecoveryWorkflowContext, AudioShardRecoveryMergeStageInput>
    for MergeAudioShardRecoveryStage
{
    type Output = AudioShardRecoveryMergeStageOutput;
    type Error = String;

    fn id(&self) -> &'static str {
        AUDIO_RECOVERY_MERGE_STAGE_ID
    }

    fn input_facts(&self, input: &AudioShardRecoveryMergeStageInput) -> WorkflowStageFacts {
        WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_result", "v1")
            .with_item_count(input.base_response.results.len())
    }

    fn output_facts(&self, _output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("AudioShardRecoveryMergeStageOutput")
    }

    async fn run(
        &self,
        _context: &mut AudioShardRecoveryWorkflowContext,
        input: AudioShardRecoveryMergeStageInput,
    ) -> Result<Self::Output, String> {
        let (merge_report, patch_gate_report) =
            if let Some(recovery_response) = input.recovery_response {
                input.base_response.merge_with_recovery_for_inputs(
                    input.base_inputs.as_slice(),
                    input.recovery_inputs.as_slice(),
                    &recovery_response,
                    input.patch_options,
                )?
            } else {
                (
                    input
                        .base_response
                        .merge_for_inputs(input.base_inputs.as_slice())?,
                    empty_patch_gate_report(),
                )
            };
        Ok(AudioShardRecoveryMergeStageOutput {
            merge_report,
            patch_gate_report,
        })
    }
}

fn build_prepared_inputs(
    materialized_shards: Vec<AudioShardMaterializedItem>,
    profile: &AudioShardWorkerProfile,
) -> Result<AudioShardPreparedInputs, String> {
    let inputs = build_audio_shard_inputs(materialized_shards.as_slice(), profile);
    let input_batch = build_audio_shard_input_batch(inputs.as_slice())?;
    Ok(AudioShardPreparedInputs {
        inputs,
        input_batch,
    })
}

async fn request_with_result_batch(
    client: &AudioShardFlightClient,
    inputs: Vec<AudioShardInput>,
    worker_budget: Option<usize>,
) -> Result<AudioShardRecoveryFlightExchange, String> {
    let response = client
        .request_with_worker_budget(inputs.as_slice(), worker_budget)
        .await?;
    let result_batch = build_audio_shard_result_batch(response.results.as_slice())?;
    Ok(AudioShardRecoveryFlightExchange {
        response,
        result_batch,
    })
}

fn audio_shard_recovery_workflow_topology() -> Result<WorkflowTopology, String> {
    let stages = vec![
        WorkflowStageBinding::required(AUDIO_BASE_MATERIALIZE_STAGE_ID),
        WorkflowStageBinding::required(AUDIO_BASE_BUILD_ROWS_STAGE_ID),
        WorkflowStageBinding::required(AUDIO_BASE_CALL_FLIGHT_STAGE_ID),
        WorkflowStageBinding::required(AUDIO_PLAN_RECOVERY_STAGE_ID),
        WorkflowStageBinding::optional(AUDIO_RECOVERY_MATERIALIZE_STAGE_ID),
        WorkflowStageBinding::optional(AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID),
        WorkflowStageBinding::optional(AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID),
        WorkflowStageBinding::required(AUDIO_RECOVERY_MERGE_STAGE_ID),
    ];
    let edges = vec![
        WorkflowTopologyEdge::new(
            AUDIO_BASE_MATERIALIZE_STAGE_ID,
            AUDIO_BASE_BUILD_ROWS_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_BASE_BUILD_ROWS_STAGE_ID,
            AUDIO_BASE_CALL_FLIGHT_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_BASE_CALL_FLIGHT_STAGE_ID,
            AUDIO_PLAN_RECOVERY_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_PLAN_RECOVERY_STAGE_ID,
            AUDIO_RECOVERY_MATERIALIZE_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_RECOVERY_MATERIALIZE_STAGE_ID,
            AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID,
            AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(
            AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID,
            AUDIO_RECOVERY_MERGE_STAGE_ID,
        ),
        WorkflowTopologyEdge::new(AUDIO_PLAN_RECOVERY_STAGE_ID, AUDIO_RECOVERY_MERGE_STAGE_ID),
    ];
    let topology = WorkflowTopology::new(AUDIO_RECOVERY_WORKFLOW_ID, stages, edges);
    topology.validate().map_err(|error| error.to_string())?;
    Ok(topology)
}
