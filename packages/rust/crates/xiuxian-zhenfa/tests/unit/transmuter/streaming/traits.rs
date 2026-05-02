use crate::transmuter::streaming::traits::{
    accumulate_text, is_ignorable_line, strip_ndjson_prefix,
};

#[test]
fn trait_helpers_accumulate_and_strip_text() {
    let mut buffer = String::new();
    accumulate_text(&mut buffer, "hello");
    accumulate_text(&mut buffer, " world");

    assert_eq!(buffer, "hello world");
    assert_eq!(strip_ndjson_prefix("data: {\"x\":1}"), "{\"x\":1}");
    assert_eq!(strip_ndjson_prefix("{\"x\":1}"), "{\"x\":1}");
}

#[test]
fn trait_helpers_identify_ignorable_lines() {
    assert!(is_ignorable_line(""));
    assert!(is_ignorable_line(":"));
    assert!(is_ignorable_line("// keep-alive"));
    assert!(!is_ignorable_line("data: {\"type\":\"chunk\"}"));
}
