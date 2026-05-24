use crate::{
    AdmissionDecision, AudioScheduleAction, AudioScheduleReason, AudioSchedulingInput,
    LaneCapability, PolyglotLane, PressureLevel, QueueReason, WorkerPressureEvidence,
};

#[test]
fn audio_schedule_uses_system_cap_for_automatic_budget() {
    let pressure = WorkerPressureEvidence::audio_shard_transcription()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = AudioSchedulingInput::audio_shards(pressure)
        .with_worker_request(None, Some(12))
        .with_shard_count(10)
        .plan();

    assert_eq!(plan.capability, LaneCapability::AudioShardTranscription);
    assert_eq!(plan.action, AudioScheduleAction::Dispatch);
    assert_eq!(plan.reason, AudioScheduleReason::CapacityAvailable);
    assert_eq!(plan.pressure, PressureLevel::Low);
    assert_eq!(plan.recommended_workers, 4);
    assert_eq!(plan.shard_wave_size, 4);
}

#[test]
fn audio_schedule_expands_initial_budget_for_large_hosted_batches() {
    let pressure = WorkerPressureEvidence::audio_shard_transcription()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = AudioSchedulingInput::audio_shards(pressure)
        .with_worker_request(None, Some(12))
        .with_shard_count(20)
        .plan();

    assert_eq!(plan.action, AudioScheduleAction::Dispatch);
    assert_eq!(plan.recommended_workers, 7);
    assert_eq!(plan.shard_wave_size, 7);
}

#[test]
fn audio_schedule_respects_owner_adaptive_budget() {
    let pressure = WorkerPressureEvidence::audio_shard_transcription()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = AudioSchedulingInput::audio_shards(pressure)
        .with_adaptive_worker_budget(Some(2))
        .with_worker_request(None, Some(12))
        .with_shard_count(10)
        .plan();

    assert_eq!(plan.action, AudioScheduleAction::Dispatch);
    assert_eq!(plan.recommended_workers, 2);
    assert_eq!(plan.shard_wave_size, 2);
}

#[test]
fn audio_schedule_caps_explicit_override_by_adaptive_budget() {
    let pressure = WorkerPressureEvidence::audio_shard_transcription()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = AudioSchedulingInput::audio_shards(pressure)
        .with_adaptive_worker_budget(Some(3))
        .with_worker_request(Some(8), Some(12))
        .with_shard_count(10)
        .plan();

    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}

#[test]
fn audio_schedule_queues_at_capacity() {
    let pressure = WorkerPressureEvidence::audio_shard_transcription()
        .with_worker_budget(Some(4), 4)
        .with_queue_depth(0);

    let plan = AudioSchedulingInput::audio_shards(pressure)
        .with_worker_request(None, Some(4))
        .with_shard_count(8)
        .plan();

    assert_eq!(plan.action, AudioScheduleAction::Queue);
    assert_eq!(plan.reason, AudioScheduleReason::AtCapacity);
    assert_eq!(
        plan.admission,
        AdmissionDecision::Queue {
            lane: PolyglotLane::PythonDocling,
            reason: QueueReason::AtCapacity,
            queue_depth: 0,
        }
    );
}
