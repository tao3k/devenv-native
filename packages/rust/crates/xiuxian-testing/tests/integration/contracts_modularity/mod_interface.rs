use super::*;

#[test]
fn modularity_pack_flags_mod_rs_with_implementation_logic() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser;
pub use parser::Parser;

pub fn parse() {}
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(findings.iter().any(|finding| finding.rule_id == "MOD-R001"));
}

#[test]
fn modularity_pack_flags_inline_module_body_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser {
    pub(crate) struct Parser;
}

pub use parser::Parser;
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 finding, got {findings:#?}"));
    assert!(
        finding.summary.contains("inline module `parser`"),
        "expected inline-module summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_flags_private_use_import_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser;
use parser::Parser;
pub use parser::Parser as PublicParser;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 finding, got {findings:#?}"));
    assert!(
        finding.summary.contains("private `use` import"),
        "expected private-use summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_flags_glob_reexport_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser;
pub use parser::*;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 finding, got {findings:#?}"));
    assert!(
        finding.summary.contains("glob re-export"),
        "expected glob-reexport summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_flags_public_module_declaration_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
pub mod parser;
pub use parser::Parser;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 finding, got {findings:#?}"));
    assert!(
        finding
            .summary
            .contains("visible module declaration `parser`"),
        "expected public-module summary, got {finding:#?}"
    );
    assert!(
        finding.remediation.contains("private `#[path ="),
        "expected path-mount remediation hint, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_flags_restricted_visible_module_declaration_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
pub(crate) mod parser;
pub(crate) use parser::Parser;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 finding, got {findings:#?}"));
    assert!(
        finding
            .summary
            .contains("visible module declaration `parser`"),
        "expected restricted-visible-module summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_flags_unparseable_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser;

pub fn parse( {
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R001")
        .unwrap_or_else(|| panic!("expected MOD-R001 parse finding, got {findings:#?}"));
    assert!(
        finding
            .summary
            .contains("could not be parsed as Rust syntax"),
        "expected parse-failure summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_interface_only_and_documented_error_contracts() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
mod parser;
pub(crate) use parser::Parser;
",
    );
    write_rust_file(
        &src_root,
        "api.rs",
        r"
/// Execute the operation.
///
/// # Errors
/// Returns an error when upstream resolution fails.
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
",
    );
    write_rust_file(
        &src_root,
        "internal/state.rs",
        r"
pub(crate) struct InternalState {
    value: usize,
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(findings.is_empty());
}

#[test]
fn modularity_pack_accepts_multiline_interface_exports_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r"
#![allow(dead_code)]
mod parser;
mod scanner;

pub use self::{
    parser::Parser,
    scanner::Scanner,
};
pub(super) use self::parser::Parser as InternalParser;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/scanner.rs",
        "pub(crate) struct Scanner;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R001"),
        "expected no MOD-R001 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_explicit_restricted_reexports_in_mod_rs() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/mod.rs",
        r#"
#[cfg(test)]
#[path = "../../tests/unit/feature/mod.rs"]
mod tests;
mod parser;
mod scanner;

pub(crate) use self::parser::Parser;
pub(super) use self::scanner::Scanner as InternalScanner;
"#,
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/scanner.rs",
        "pub(crate) struct Scanner;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R001"),
        "expected no MOD-R001 finding, got {findings:#?}"
    );
}
