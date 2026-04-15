use super::*;

#[test]
fn modularity_pack_flags_root_entry_from_internal_module() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod internal;
mod parser;
mod service;

pub(crate) use self::internal::FeatureState;
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
        "feature/service.rs",
        "pub(crate) struct Service;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R012")
        .unwrap_or_else(|| panic!("expected MOD-R012 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("helper modules"),
        "expected root-owner title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_root_entry_from_canonical_module() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub(crate) use self::service::Service;
pub(crate) use self::runtime::Runtime;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R012"),
        "expected no MOD-R012 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_visible_child_module_in_root_facade() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
pub(crate) mod service;
mod runtime;

pub(crate) use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub(crate) struct Parser;");
    write_rust_file(
        &src_root,
        "feature/service.rs",
        "pub(crate) struct Service;",
    );
    write_rust_file(
        &src_root,
        "feature/runtime.rs",
        "pub(crate) struct Runtime;",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R013")
        .unwrap_or_else(|| panic!("expected MOD-R013 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("child modules private"),
        "expected root-child-visibility title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_private_child_modules_in_root_facade() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R013"),
        "expected no MOD-R013 finding, got {findings:#?}"
    );
}
