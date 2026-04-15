use super::*;

#[test]
fn modularity_pack_flags_helper_bucket_as_secondary_root_doc_seam() {
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
//! Start in `service`; internal handles glue.

mod internal;
mod parser;
mod runtime;
mod service;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(
        &src_root,
        "feature/internal.rs",
        "pub(crate) struct FeatureState;",
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
        .find(|finding| finding.rule_id == "MOD-R023")
        .unwrap_or_else(|| panic!("expected MOD-R023 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("helper-bucket secondary seams"),
        "expected root-doc-secondary-owner title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_canonical_secondary_root_doc_seam() {
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

mod internal;
mod parser;
mod runtime;
mod service;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(
        &src_root,
        "feature/internal.rs",
        "pub(crate) struct FeatureState;",
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R023"),
        "expected no MOD-R023 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_helper_bucket_secondary_doc_when_parent_is_public() {
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
//! Start in `service`; internal handles glue.

mod internal;
mod parser;
mod runtime;
mod service;

pub use self::service::Service;
",
    );
    write_rust_file(
        &src_root,
        "feature/internal.rs",
        "pub(crate) struct FeatureState;",
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R023"),
        "expected no MOD-R023 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_alias_named_canonical_owner_in_root_doc() {
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
//! Start in `FeatureService`.

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
        .find(|finding| finding.rule_id == "MOD-R024")
        .unwrap_or_else(|| panic!("expected MOD-R024 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("canonical owner module directly"),
        "expected root-doc-owner-name title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_real_module_name_for_canonical_owner_hint() {
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
//! Start in `service`.

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
        findings.iter().all(|finding| finding.rule_id != "MOD-R024"),
        "expected no MOD-R024 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_alias_named_owner_hint_when_parent_is_public() {
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
//! Start in `FeatureService`.

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
        findings.iter().all(|finding| finding.rule_id != "MOD-R024"),
        "expected no MOD-R024 finding, got {findings:#?}"
    );
}
