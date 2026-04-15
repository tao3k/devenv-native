use super::*;

#[test]
fn modularity_pack_flags_root_seam_without_navigation_hint() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;
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
        .find(|finding| finding.rule_id == "MOD-R011")
        .unwrap_or_else(|| panic!("expected MOD-R011 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("navigation hint"),
        "expected root-hint title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_root_seam_with_doc_hint() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Parser + runtime seam for the feature.

mod parser;
mod runtime;
mod service;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R011"),
        "expected no MOD-R011 finding, got {findings:#?}"
    );
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R014"),
        "expected no MOD-R014 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_generic_doc_only_root_hint() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Feature seam for the demo.

mod parser;
mod runtime;
mod service;
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
        .find(|finding| finding.rule_id == "MOD-R014")
        .unwrap_or_else(|| panic!("expected MOD-R014 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("name a child module"),
        "expected root-doc-hint title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_doc_only_root_hint_that_names_child_module() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`, then descend into `parser` for syntax work.

mod parser;
mod runtime;
mod service;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R014"),
        "expected no MOD-R014 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_unfocused_root_entry_surface() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser;
pub(crate) use self::runtime::Runtime;
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
        .find(|finding| finding.rule_id == "MOD-R015")
        .unwrap_or_else(|| panic!("expected MOD-R015 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("primary entry owner"),
        "expected root-entry-focus title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_root_entry_surface_with_named_primary_owner() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`; `parser` and `runtime` support the seam.

mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser;
pub(crate) use self::runtime::Runtime;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R015"),
        "expected no MOD-R015 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_root_doc_owner_not_present_in_visible_entries() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`.

mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser;
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
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R016")
        .unwrap_or_else(|| panic!("expected MOD-R016 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("align with visible entry surface"),
        "expected root-doc-owner-alignment title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_root_doc_owner_present_in_visible_entries() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`.

mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R016"),
        "expected no MOD-R016 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_root_owner_convergence_drift() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`.

mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::{ParsePlan, Parser};
pub(crate) use self::service::Service;
",
    );
    write_rust_file(
        &src_root,
        "feature/parser.rs",
        r"
pub(crate) struct Parser;
pub(crate) struct ParsePlan;
",
    );
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
        .find(|finding| finding.rule_id == "MOD-R017")
        .unwrap_or_else(|| panic!("expected MOD-R017 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("converge on one owner"),
        "expected root-owner-convergence title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_converged_root_owner_surface() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
//! Start in `service`.

mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser;
pub(crate) use self::service::{Service, ServicePlan};
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
        r"
pub(crate) struct Service;
pub(crate) struct ServicePlan;
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R017"),
        "expected no MOD-R017 finding, got {findings:#?}"
    );
}
