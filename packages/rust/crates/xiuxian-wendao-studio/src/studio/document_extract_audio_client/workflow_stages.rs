//! Stage implementations for the audio recovery workflow.

use std::collections::HashMap;

use xiuxian_qianji::{WorkflowStage, WorkflowStageFacts};
use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardMaterializedItem, AudioShardPlan, AudioShardWorkerProfile,
    build_audio_shard_input_batch, build_audio_shard_inputs, build_audio_shard_result_batch,
    combine_admitted_and_fresh_audio_transcripts, lookup_audio_transcript_admission,
    lookup_planned_audio_transcript_admission, materialize_audio_shard_manifests,
    persist_audio_transcript_admission, plan_audio_shards,
};

use super::{
    AUDIO_BASE_BUILD_ROWS_STAGE_ID, AUDIO_BASE_CALL_FLIGHT_STAGE_ID,
    AUDIO_BASE_MATERIALIZE_STAGE_ID, AUDIO_PLAN_RECOVERY_STAGE_ID,
    AUDIO_RECOVERY_BUILD_ROWS_STAGE_ID, AUDIO_RECOVERY_CALL_FLIGHT_STAGE_ID,
    AUDIO_RECOVERY_MATERIALIZE_STAGE_ID, AUDIO_RECOVERY_MERGE_STAGE_ID, AudioShardPreparedInputs,
    AudioShardRecoveryFlightExchange, AudioShardRecoveryMergeStageInput,
    AudioShardRecoveryMergeStageOutput, AudioShardRecoveryPlanningStageInput,
    AudioShardRecoveryWorkflowContext,
};
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightRequestOptions, AudioShardFlightResponse,
    AudioShardRecoveryPlanRequest, AudioShardRecoveryPlanning, empty_patch_gate_report,
};
use xiuxian_wendao_attachments::audio::AudioPlannedTranscriptAdmissionLookup;

#[derive(Debug, Clone, Copy)]
pub(super) struct BaseMaterializeAudioShardPlanStage;

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
        let request_options = context
            .request_options
            .with_worker_budget(context.base_worker_budget);
        let manifests = plan_audio_shards(&input)?;
        let admission_options = request_options.transcript_admission_options();
        let preflight = lookup_planned_audio_transcript_admission(
            manifests.as_slice(),
            &context.profile,
            &admission_options,
        )?;
        if preflight.all_hit {
            context.base_planned_result_preflight = Some(preflight);
            return Ok(Vec::new());
        }
        let materialized = materialize_audio_shard_manifests(
            &input,
            &context.materialization,
            preflight.miss_manifests.as_slice(),
        )?;
        context.base_planned_result_preflight = Some(preflight);
        Ok(materialized)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RecoveryMaterializeAudioShardPlanStage;

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
        let request_options = context
            .request_options
            .with_worker_budget(context.recovery_worker_budget);
        let manifests = plan_audio_shards(&input)?;
        let admission_options = request_options.transcript_admission_options();
        let preflight = lookup_planned_audio_transcript_admission(
            manifests.as_slice(),
            &context.profile,
            &admission_options,
        )?;
        if preflight.all_hit {
            context.recovery_planned_result_preflight = Some(preflight);
            return Ok(Vec::new());
        }
        let materialized = materialize_audio_shard_manifests(
            &input,
            &context.materialization,
            preflight.miss_manifests.as_slice(),
        )?;
        context.recovery_planned_result_preflight = Some(preflight);
        Ok(materialized)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BaseBuildAudioShardInputsStage;

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
        if let Some(preflight) = context.base_planned_result_preflight.as_ref() {
            return build_prepared_inputs_with_planned_hits(
                input.as_slice(),
                &context.profile,
                preflight,
            );
        }
        build_prepared_inputs(input.as_slice(), &context.profile)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RecoveryBuildAudioShardInputsStage;

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
        if let Some(preflight) = context.recovery_planned_result_preflight.as_ref() {
            return build_prepared_inputs_with_planned_hits(
                input.as_slice(),
                &context.profile,
                preflight,
            );
        }
        build_prepared_inputs(input.as_slice(), &context.profile)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BaseRequestAudioShardFlightStage;

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
        if let Some(preflight) = context.base_planned_result_preflight.take() {
            return request_with_planned_preflight(
                &context.client,
                input,
                preflight,
                context.base_worker_budget,
                &context.request_options,
            )
            .await;
        }
        request_with_result_batch(
            &context.client,
            input.inputs,
            context.base_worker_budget,
            &context.request_options,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RecoveryRequestAudioShardFlightStage;

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
        if let Some(preflight) = context.recovery_planned_result_preflight.take() {
            return request_with_planned_preflight(
                &context.client,
                input,
                preflight,
                context.recovery_worker_budget,
                &context.request_options,
            )
            .await;
        }
        request_with_result_batch(
            &context.client,
            input.inputs,
            context.recovery_worker_budget,
            &context.request_options,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PlanAudioShardRecoveryStage;

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
pub(super) struct MergeAudioShardRecoveryStage;

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
    materialized_shards: &[AudioShardMaterializedItem],
    profile: &AudioShardWorkerProfile,
) -> Result<AudioShardPreparedInputs, String> {
    let inputs = build_audio_shard_inputs(materialized_shards, profile);
    build_prepared_input_rows(inputs.as_slice())
}

fn build_prepared_inputs_with_planned_hits(
    materialized_shards: &[AudioShardMaterializedItem],
    profile: &AudioShardWorkerProfile,
    preflight: &AudioPlannedTranscriptAdmissionLookup,
) -> Result<AudioShardPreparedInputs, String> {
    let mut inputs = preflight.inputs.clone();
    inputs.extend(build_audio_shard_inputs(materialized_shards, profile));
    inputs.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });
    build_prepared_input_rows(inputs.as_slice())
}

fn build_prepared_input_rows(
    inputs: &[AudioShardInput],
) -> Result<AudioShardPreparedInputs, String> {
    let input_batch = build_audio_shard_input_batch(inputs)?;
    Ok(AudioShardPreparedInputs {
        inputs: inputs.to_vec(),
        input_batch,
    })
}

fn planned_preflight_exchange(
    preflight: AudioPlannedTranscriptAdmissionLookup,
) -> Result<AudioShardRecoveryFlightExchange, String> {
    let response = AudioShardFlightResponse {
        results: preflight.results,
    };
    let result_batch = build_audio_shard_result_batch(response.results.as_slice())?;
    Ok(AudioShardRecoveryFlightExchange {
        response,
        result_batch,
        transcript_admission_stats: preflight.stats,
    })
}

async fn request_with_planned_preflight(
    client: &AudioShardFlightClient,
    prepared_inputs: AudioShardPreparedInputs,
    preflight: AudioPlannedTranscriptAdmissionLookup,
    worker_budget: Option<usize>,
    request_options: &AudioShardFlightRequestOptions,
) -> Result<AudioShardRecoveryFlightExchange, String> {
    if preflight.all_hit {
        return planned_preflight_exchange(preflight);
    }
    let admitted_results = preflight
        .results
        .iter()
        .map(|result| (result.shard_element_id.clone(), result.clone()))
        .collect::<HashMap<_, _>>();
    let miss_inputs = prepared_inputs
        .inputs
        .iter()
        .filter(|input| !admitted_results.contains_key(input.shard_element_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let fresh_exchange =
        request_with_result_batch(client, miss_inputs, worker_budget, request_options).await?;
    let mut transcript_admission_stats = preflight.stats;
    transcript_admission_stats.add_assign(&fresh_exchange.transcript_admission_stats);
    let response = AudioShardFlightResponse {
        results: combine_admitted_and_fresh_audio_transcripts(
            prepared_inputs.inputs.as_slice(),
            &admitted_results,
            fresh_exchange.response.results.as_slice(),
        ),
    };
    let result_batch = build_audio_shard_result_batch(response.results.as_slice())?;
    Ok(AudioShardRecoveryFlightExchange {
        response,
        result_batch,
        transcript_admission_stats,
    })
}

async fn request_with_result_batch(
    client: &AudioShardFlightClient,
    inputs: Vec<AudioShardInput>,
    worker_budget: Option<usize>,
    request_options: &AudioShardFlightRequestOptions,
) -> Result<AudioShardRecoveryFlightExchange, String> {
    let request_options = request_options.with_worker_budget(worker_budget);
    let admission_options = request_options.transcript_admission_options();
    let cache_lookup = lookup_audio_transcript_admission(inputs.as_slice(), &admission_options)?;
    let fresh_response = if cache_lookup.miss_inputs.is_empty() {
        AudioShardFlightResponse {
            results: Vec::new(),
        }
    } else {
        client
            .request_with_options(cache_lookup.miss_inputs.as_slice(), &request_options)
            .await?
    };
    let persist_stats = persist_audio_transcript_admission(
        cache_lookup.miss_inputs.as_slice(),
        fresh_response.results.as_slice(),
        &admission_options,
    )?;
    let mut transcript_admission_stats = cache_lookup.stats;
    transcript_admission_stats.add_assign(&persist_stats);
    let response = AudioShardFlightResponse {
        results: combine_admitted_and_fresh_audio_transcripts(
            inputs.as_slice(),
            &cache_lookup.admitted_results,
            fresh_response.results.as_slice(),
        ),
    };
    let result_batch = build_audio_shard_result_batch(response.results.as_slice())?;
    Ok(AudioShardRecoveryFlightExchange {
        response,
        result_batch,
        transcript_admission_stats,
    })
}
