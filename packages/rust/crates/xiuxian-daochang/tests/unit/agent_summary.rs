//! Test coverage for xiuxian-daochang behavior.

use xiuxian_daochang::{DrainedTurn, summarise_drained_turns};

#[test]
fn summarise_drained_turns_intent_first_user() {
    let drained = vec![
        DrainedTurn::new("user", "what is 2+2?", 0),
        DrainedTurn::new("assistant", "4", 0),
    ];
    let summary = summarise_drained_turns(&drained);
    assert_eq!(summary.intent, "what is 2+2?");
    assert_eq!(summary.experience, "4");
    assert_eq!(summary.outcome, "completed");
}

#[test]
fn summarise_drained_turns_outcome_error() {
    let drained = vec![
        DrainedTurn::new("user", "run tool", 0),
        DrainedTurn::new("assistant", "Error: connection failed", 1),
    ];
    let summary = summarise_drained_turns(&drained);
    assert_eq!(summary.outcome, "error");
}

#[test]
fn summarise_drained_turns_no_user_fallback() {
    let drained = vec![DrainedTurn::new("assistant", "ok", 0)];
    let summary = summarise_drained_turns(&drained);
    assert_eq!(summary.intent, "(no user message)");
    assert_eq!(summary.experience, "ok");
    assert_eq!(summary.outcome, "completed");
}
