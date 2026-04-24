use crate::parsers::zhixing::tasks::{normalize_identity_token, parse_task_projection};
use crate::skill_runtime::zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
};

#[test]
fn parse_task_projection_extracts_task_metadata_comment() {
    let line = format!(
        "- [x] Ship parser lane <!-- id: parser-lane, priority: high, {ATTR_JOURNAL_CARRYOVER}: 2, {ATTR_TIMER_SCHEDULED}: 2026-04-06T09:00:00Z, {ATTR_TIMER_REMINDED}: yes -->"
    );
    let parsed = parse_task_projection(&line, 7).unwrap_or_else(|| panic!("task should parse"));

    assert_eq!(parsed.title, "Ship parser lane");
    assert_eq!(parsed.line_no, 7);
    assert!(parsed.is_completed);
    assert_eq!(parsed.task_id.as_deref(), Some("parser-lane"));
    assert_eq!(parsed.priority.as_deref(), Some("high"));
    assert_eq!(parsed.carryover, 2);
    assert_eq!(parsed.scheduled_at.as_deref(), Some("2026-04-06T09:00:00Z"));
    assert_eq!(parsed.reminded, Some(true));
}

#[test]
fn parse_task_projection_falls_back_to_inline_carryover_marker() {
    let line = "- [ ] Follow up journal:carryover:3";
    let parsed = parse_task_projection(line, 2).unwrap_or_else(|| panic!("task should parse"));

    assert_eq!(parsed.title, "Follow up journal:carryover:3");
    assert_eq!(parsed.carryover, 3);
    assert!(!parsed.is_completed);
}

#[test]
fn parse_task_projection_rejects_empty_titles() {
    assert!(parse_task_projection("- [ ]    ", 3).is_none());
}

#[test]
fn normalize_identity_token_normalizes_and_falls_back() {
    assert_eq!(normalize_identity_token(" Task #42 "), "task--42");
    assert_eq!(normalize_identity_token("!!!"), "unknown");
}
