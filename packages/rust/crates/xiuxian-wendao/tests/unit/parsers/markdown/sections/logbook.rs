use super::support::extract_sections_from;
use crate::parsers::markdown::sections::{extract_logbook_entries, parse_logbook_entry};

#[test]
fn test_parse_logbook_entry_valid() {
    let line = "- [2025-03-14] Agent Started: Initiating structural audit.";
    let entry = parse_logbook_entry(line, 1);
    assert!(entry.is_some());
    let Some(entry) = entry else {
        panic!("expected valid logbook entry");
    };
    assert_eq!(entry.timestamp, "2025-03-14");
    assert_eq!(entry.message, "Agent Started: Initiating structural audit.");
    assert_eq!(entry.line_number, 1);
}

#[test]
fn test_parse_logbook_entry_with_brackets_in_message() {
    let line = "- [2025-03-14] Step [audit] completed with status OK.";
    let entry = parse_logbook_entry(line, 2);
    assert!(entry.is_some());
    let Some(entry) = entry else {
        panic!("expected valid logbook entry with brackets");
    };
    assert_eq!(entry.timestamp, "2025-03-14");
    assert_eq!(entry.message, "Step [audit] completed with status OK.");
}

#[test]
fn test_parse_logbook_entry_invalid_format() {
    assert!(parse_logbook_entry("[2025-03-14] Message", 1).is_none());
    assert!(parse_logbook_entry("- 2025-03-14 Message", 1).is_none());
    assert!(parse_logbook_entry("- [2025-03-14] ", 1).is_none());
    assert!(parse_logbook_entry("- [] Message", 1).is_none());
}

#[test]
fn test_extract_logbook_entries_basic() {
    let lines = vec![
        ":LOGBOOK:".to_string(),
        "- [2025-03-14] Agent Started: Initiating structural audit.".to_string(),
        "- [2025-03-14] Step [audit] completed with status OK.".to_string(),
        ":END:".to_string(),
        "Content after logbook.".to_string(),
    ];
    let entries = extract_logbook_entries(&lines, 1);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].timestamp, "2025-03-14");
    assert_eq!(
        entries[0].message,
        "Agent Started: Initiating structural audit."
    );
    assert_eq!(entries[1].message, "Step [audit] completed with status OK.");
}

#[test]
fn test_extract_logbook_entries_empty() {
    let lines = vec![":LOGBOOK:".to_string(), ":END:".to_string()];
    let entries = extract_logbook_entries(&lines, 1);
    assert!(entries.is_empty());
}

#[test]
fn test_extract_logbook_entries_no_block() {
    let lines = vec![
        "- [2025-03-14] This is not in a logbook block.".to_string(),
        "Just some content.".to_string(),
    ];
    let entries = extract_logbook_entries(&lines, 1);
    assert!(entries.is_empty());
}

#[test]
fn test_extract_sections_with_logbook() {
    let body = r"# Task: Refactor Authentication
:PROPERTIES:
:ID:       task-auth-001
:STATUS:   RUNNING
:END:

:LOGBOOK:
- [2025-03-14] Agent Started: Initiating structural audit.
- [2025-03-14] Step [audit] completed with status OK.
:END:

Some task content here.
";
    let sections = extract_sections_from(body);

    assert_eq!(sections.len(), 1);
    let section = &sections[0];
    assert_eq!(
        section.attributes.get("ID"),
        Some(&"task-auth-001".to_string())
    );
    assert_eq!(
        section.attributes.get("STATUS"),
        Some(&"RUNNING".to_string())
    );
    assert_eq!(section.logbook.len(), 2);
    assert_eq!(section.logbook[0].timestamp, "2025-03-14");
    assert_eq!(
        section.logbook[0].message,
        "Agent Started: Initiating structural audit."
    );
    assert_eq!(
        section.logbook[1].message,
        "Step [audit] completed with status OK."
    );
}
