use super::*;

#[test]
fn modularity_pack_flags_plain_pub_entry_in_internal_root_seam() {
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
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R018")
        .unwrap_or_else(|| panic!("expected MOD-R018 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("restricted entry visibility"),
        "expected root-entry-visibility title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_restricted_entry_in_internal_root_seam() {
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R018"),
        "expected no MOD-R018 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_plain_pub_entry_when_parent_module_is_public() {
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R018"),
        "expected no MOD-R018 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_internal_root_seam_with_multiple_visible_owners() {
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
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R019")
        .unwrap_or_else(|| panic!("expected MOD-R019 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("canonical visible owner"),
        "expected root-entry-curation title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_internal_root_seam_with_one_visible_owner() {
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
//! Start in `service`; parser and runtime stay leaf-owned.

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
        findings.iter().all(|finding| finding.rule_id != "MOD-R019"),
        "expected no MOD-R019 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_public_root_seam_with_multiple_visible_owners() {
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
mod parser;
mod runtime;
mod service;

pub use self::parser::Parser;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R019"),
        "expected no MOD-R019 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_inventory_style_root_doc_for_internal_canonical_owner() {
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
        .find(|finding| finding.rule_id == "MOD-R020")
        .unwrap_or_else(|| panic!("expected MOD-R020 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("canonical owner"),
        "expected root-doc-curation title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_focused_root_doc_for_internal_canonical_owner() {
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
//! Start in `service`; parser stays leaf-owned.

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
        findings.iter().all(|finding| finding.rule_id != "MOD-R020"),
        "expected no MOD-R020 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_accepts_inventory_style_root_doc_for_public_parent() {
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R020"),
        "expected no MOD-R020 finding, got {findings:#?}"
    );
}
