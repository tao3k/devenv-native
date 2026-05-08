//! Integration tests for `SessionWindow`.

use xiuxian_window::{SessionWindow, SessionWindowCheckpointId};

#[test]
fn test_append_and_get_recent() {
    let mut w = SessionWindow::new("s1", 10);
    w.append_turn("user", "hello", 0, None);
    w.append_turn("assistant", "hi", 1, None);
    let recent = w.get_recent_turns(5);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].role, "user");
    assert_eq!(recent[0].content, "hello");
    assert_eq!(recent[1].tool_count, 1);
}

#[test]
fn test_get_stats() {
    let mut w = SessionWindow::new("s1", 100);
    w.append_turn("user", "a", 0, None);
    w.append_turn("assistant", "b", 2, None);
    let stats = w.get_stats();
    assert_eq!(stats.total_turns, 2);
    assert_eq!(stats.total_tool_calls, 2);
    assert_eq!(stats.window_used, 2);
}

#[test]
fn test_checkpoint_attached() {
    let mut w = SessionWindow::new("s1", 5);
    w.append_turn(
        "assistant",
        "checkpoint",
        0,
        Some(SessionWindowCheckpointId::from("cp1")),
    );
    let recent = w.get_recent_turns(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].checkpoint_id.as_deref(), Some("cp1"));
}

#[test]
fn test_tool_count_trim_and_drain() {
    let mut w = SessionWindow::new("s1", 3);
    w.append_turn("user", "t1", 1, None);
    w.append_turn("assistant", "t2", 2, None);
    w.append_turn("assistant", "t3", 3, None);
    w.append_turn("assistant", "t4", 4, None);

    let stats = w.get_stats();
    assert_eq!(stats.total_turns, 3);
    assert_eq!(stats.total_tool_calls, 9);

    let drained = w.drain_oldest_turns(2);
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].content, "t2");
    assert_eq!(drained[1].content, "t3");

    let stats = w.get_stats();
    assert_eq!(stats.total_turns, 1);
    assert_eq!(stats.total_tool_calls, 4);
}

#[test]
fn test_zero_capacity_window() {
    let mut w = SessionWindow::new("s1", 0);
    w.append_turn("user", "a", 3, None);
    let stats = w.get_stats();
    assert_eq!(stats.total_turns, 0);
    assert_eq!(stats.total_tool_calls, 0);
    assert!(w.get_recent_turns(1).is_empty());
}

#[test]
fn test_max_turns_trim() {
    let mut w = SessionWindow::new("s1", 3);
    for i in 0..5 {
        w.append_turn("user", &i.to_string(), 0, None);
    }
    let stats = w.get_stats();
    assert_eq!(stats.total_turns, 3);
    let recent = w.get_recent_turns(10);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].content, "2");
}
