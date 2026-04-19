//! `QTable` tests.

use std::sync::Arc;
use std::thread;

use xiuxian_memory_engine::QTable;

#[test]
fn test_q_table_basic_update() {
    let q_table = QTable::new();

    assert!((q_table.get_q("ep-001") - 0.5).abs() < f32::EPSILON);

    let q_new = q_table.update("ep-001", 1.0);
    assert!((q_new - 0.6).abs() < f32::EPSILON);

    assert!((q_table.get_q("ep-001") - 0.6).abs() < f32::EPSILON);
}

#[test]
fn test_q_table_negative_reward() {
    let q_table = QTable::new();

    let q_new = q_table.update("ep-001", 0.0);
    assert!((q_new - 0.4).abs() < f32::EPSILON);
}

#[test]
fn test_q_table_clamping() {
    let q_table = QTable::with_params(0.5, 0.95);

    for _ in 0..10 {
        q_table.update("ep-001", 1.0);
    }

    assert!((q_table.get_q("ep-001") - 1.0).abs() < 0.01);

    for _ in 0..10 {
        q_table.update("ep-002", 0.0);
    }

    assert!((q_table.get_q("ep-002") - 0.0).abs() < 0.01);
}

#[test]
fn test_batch_update() {
    let q_table = QTable::new();

    let updates = vec![
        ("ep-001".to_string(), 1.0),
        ("ep-002".to_string(), 0.0),
        ("ep-003".to_string(), 0.5),
    ];

    let results = q_table.update_batch(&updates);

    assert_eq!(results.len(), 3);
    assert!((q_table.get_q("ep-001") - 0.6).abs() < f32::EPSILON);
    assert!((q_table.get_q("ep-002") - 0.4).abs() < f32::EPSILON);
    assert!((q_table.get_q("ep-003") - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_q_table_parallel_updates_remain_thread_safe() {
    let q_table = Arc::new(QTable::new());
    let episode_ids = ["ep-a", "ep-b", "ep-c", "ep-d"];

    let workers: Vec<_> = episode_ids
        .into_iter()
        .enumerate()
        .map(|(index, episode_id)| {
            let q_table = Arc::clone(&q_table);
            thread::spawn(move || {
                for step in 0..128 {
                    let reward = if (index + step) % 2 == 0 { 1.0 } else { 0.0 };
                    q_table.update(episode_id, reward);
                }
            })
        })
        .collect();

    for worker in workers {
        worker
            .join()
            .unwrap_or_else(|_| panic!("parallel q_table worker panicked"));
    }

    assert_eq!(q_table.len(), 4);
    for episode_id in episode_ids {
        let q_value = q_table.get_q(episode_id);
        assert!(
            (0.0..=1.0).contains(&q_value),
            "parallel update should keep Q-value clamped, got {q_value}"
        );
    }
}
