use super::*;
use std::fmt::Write;
use std::fs;

pub(super) fn write_file(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    let Some(parent) = path.parent() else {
        panic!("fixture path should have parent: {path:?}");
    };
    fs::create_dir_all(parent).unwrap_or_else(|error| panic!("parent should exist: {error}"));
    fs::write(path, content).unwrap_or_else(|error| panic!("fixture should be written: {error}"));
}

pub(super) fn make_unit_test_fixture(test_count: usize, helper_lines: usize) -> String {
    let mut content = String::from("use super::*;\n\n");
    for index in 0..helper_lines {
        let _ = writeln!(content, "const LINE_{index}: usize = {index};");
    }
    content.push('\n');
    for index in 0..test_count {
        let _ = writeln!(
            content,
            "#[test]\nfn case_{index}() {{\n    assert_eq!(LINE_0, 0);\n}}\n"
        );
    }
    content
}

pub(super) fn make_integration_test_fixture(test_count: usize, helper_lines: usize) -> String {
    let mut content = String::from("use super::*;\n\n");
    for index in 0..helper_lines {
        let _ = writeln!(content, "const CASE_LINE_{index}: usize = {index};");
    }
    content.push('\n');
    for index in 0..test_count {
        let _ = writeln!(
            content,
            "#[test]\nfn contract_case_{index}() {{\n    assert_eq!(CASE_LINE_0, 0);\n}}\n"
        );
    }
    content
}
