use super::{
    AudioShardCapacityController, AudioShardCapacitySnapshot,
    audio_capacity_latency_pressure_limit_ms,
};

#[test]
fn audio_capacity_starts_from_system_sqrt_budget() {
    let controller = AudioShardCapacityController::new(12);

    let snapshot = controller.snapshot();

    assert_eq!(snapshot.max_worker_bound, 12);
    assert_eq!(snapshot.current_worker_budget, 4);
    assert_eq!(controller.budget_for_shards(10), 4);
    assert_eq!(controller.budget_for_shards(20), 7);
}

#[test]
fn audio_capacity_budget_is_capped_by_shard_count() {
    let controller = AudioShardCapacityController::new_with_current_budget(12, 8);

    assert_eq!(controller.budget_for_shards(3), 3);
}

#[test]
fn audio_capacity_does_not_expand_large_batches_after_pressure() {
    let controller = AudioShardCapacityController::new_with_current_budget(12, 6);

    controller.record_failure();

    assert_eq!(controller.snapshot().current_worker_budget, 3);
    assert_eq!(controller.budget_for_shards(20), 3);
}

#[test]
fn audio_capacity_reduces_budget_on_failure_pressure() {
    let controller = AudioShardCapacityController::new_with_current_budget(12, 7);

    controller.record_failure();
    let snapshot = controller.snapshot();

    assert_eq!(snapshot.current_worker_budget, 4);
    assert_eq!(snapshot.healthy_streak, 0);
    assert_eq!(snapshot.budget_decrease_events, 1);
}

#[test]
fn audio_capacity_increases_after_consecutive_healthy_workflows() {
    let controller = AudioShardCapacityController::new_with_current_budget(12, 4);

    controller.record_success(3, 1_000);
    assert_eq!(controller.snapshot().current_worker_budget, 4);

    controller.record_success(3, 1_000);
    let snapshot = controller.snapshot();

    assert_eq!(snapshot.current_worker_budget, 5);
    assert_eq!(snapshot.healthy_streak, 0);
    assert_eq!(snapshot.budget_increase_events, 1);
}

#[test]
fn audio_capacity_latency_pressure_reduces_budget() {
    let controller = AudioShardCapacityController::new_with_current_budget(12, 6);

    controller.record_success(2, audio_capacity_latency_pressure_limit_ms(2) + 1);

    assert_eq!(
        controller.snapshot(),
        AudioShardCapacitySnapshot {
            max_worker_bound: 12,
            current_worker_budget: 3,
            healthy_streak: 0,
            budget_increase_events: 0,
            budget_decrease_events: 1,
        }
    );
}
