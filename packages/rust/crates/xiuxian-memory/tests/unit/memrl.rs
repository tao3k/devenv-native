//! Tests for the `MemRL` learner surface.

use xiuxian_memory::{MemRLCortex, MemoryAction, MemoryState, MemoryTaskKind};

#[test]
fn test_q_learning_evolution() {
    let mut cortex = MemRLCortex::new();

    let s1 = MemoryState {
        context_entropy: 5,
        persona_hash: 123,
        task_kind: MemoryTaskKind::new("Research"),
    };

    let action = MemoryAction::Promote;
    let reward = 10.0;

    let s2 = MemoryState {
        context_entropy: 1,
        persona_hash: 123,
        task_kind: MemoryTaskKind::new("Research"),
    };

    let initial_q = *cortex.q_table.get(&(s1.clone(), action)).unwrap_or(&0.0);
    assert!((initial_q - 0.0).abs() < f64::EPSILON);

    for _ in 0..5 {
        cortex.update(s1.clone(), action, reward, &s2);
    }

    let Some(evolved_q) = cortex.q_table.get(&(s1.clone(), action)) else {
        panic!("expected learned Q-value to be present");
    };
    let evolved_q = *evolved_q;
    assert!(evolved_q > 4.0);
    assert_eq!(cortex.decide(&s1), MemoryAction::Promote);
}
