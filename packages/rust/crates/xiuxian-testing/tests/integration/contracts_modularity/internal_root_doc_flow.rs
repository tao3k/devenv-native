use super::*;

#[test]
fn modularity_pack_flags_secondary_module_named_before_canonical_owner() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Parser handles syntax; start in `service`.

mod parser;
mod runtime;
mod service;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R021")
        .unwrap_or_else(|| panic!("expected MOD-R021 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("owner first"),
        "expected root-doc-priority title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_owner_first_root_doc_when_secondary_is_mentioned() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`; parser handles syntax.

mod parser;
mod runtime;
mod service;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R021"),
        "expected no MOD-R021 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_secondary_first_doc_when_parent_module_is_public() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
pub mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Parser handles syntax; start in `service`.

mod parser;
mod runtime;
mod service;

pub use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R021"),
        "expected no MOD-R021 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_over_budget_secondary_mentions_in_internal_root_doc() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`; parser handles syntax; runtime executes requests.

mod parser;
mod runtime;
mod service;
mod storage;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );
    write_rust_file(
        &src_root,
        "feature/storage.rs",
        "pub(crate) struct Storage;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R022")
        .unwrap_or_else(|| panic!("expected MOD-R022 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("secondary seam mentions"),
        "expected root-doc-secondary-budget title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_single_secondary_mention_in_internal_root_doc() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`; parser handles syntax.

mod parser;
mod runtime;
mod service;
mod storage;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );
    write_rust_file(
        &src_root,
        "feature/storage.rs",
        "pub(crate) struct Storage;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R022"),
        "expected no MOD-R022 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_over_budget_doc_when_parent_module_is_public() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "lib.rs",
        r"
pub mod feature;
",
    );
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`; parser handles syntax; runtime executes requests.

mod parser;
mod runtime;
mod service;
mod storage;

pub use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );
    write_rust_file(
        &src_root,
        "feature/storage.rs",
        "pub(crate) struct Storage;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R022"),
        "expected no MOD-R022 finding, got {findings:#?}"
    );
}
