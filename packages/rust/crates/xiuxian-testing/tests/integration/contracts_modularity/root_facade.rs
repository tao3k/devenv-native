use super::*;

#[test]
fn modularity_pack_flags_root_module_that_loses_toc_role() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub(crate) use parser::Parser;
pub(crate) use service::Service;

pub struct FeatureState {
    stage: usize,
}

impl FeatureState {
    pub fn new(stage: usize) -> Self {
        Self { stage }
    }
}

pub fn execute(state: &FeatureState) -> usize {
    let first = state.stage + 1;
    let second = first + 2;
    let third = second + 3;
    let fourth = third + 4;
    let fifth = fourth + 5;
    let sixth = fifth + 6;
    let seventh = sixth + 7;
    let eighth = seventh + 8;
    let ninth = eighth + 9;
    let tenth = ninth + 10;
    let eleventh = tenth + 11;
    let twelfth = eleventh + 12;
    let thirteenth = twelfth + 13;
    let fourteenth = thirteenth + 14;
    let fifteenth = fourteenth + 15;
    let sixteenth = fifteenth + 16;
    let seventeenth = sixteenth + 17;
    let eighteenth = seventeenth + 18;
    let nineteenth = eighteenth + 19;
    let twentieth = nineteenth + 20;
    let twenty_first = twentieth + 21;
    let twenty_second = twenty_first + 22;
    let twenty_third = twenty_second + 23;
    let twenty_fourth = twenty_third + 24;
    let twenty_fifth = twenty_fourth + 25;
    let twenty_sixth = twenty_fifth + 26;
    let twenty_seventh = twenty_sixth + 27;
    let twenty_eighth = twenty_seventh + 28;
    let twenty_ninth = twenty_eighth + 29;
    let thirtieth = twenty_ninth + 30;
    let thirty_first = thirtieth + 31;
    let thirty_second = thirty_first + 32;
    let thirty_third = thirty_second + 33;
    let thirty_fourth = thirty_third + 34;
    let thirty_fifth = thirty_fourth + 35;
    let thirty_sixth = thirty_fifth + 36;
    let thirty_seventh = thirty_sixth + 37;
    let thirty_eighth = thirty_seventh + 38;
    let thirty_ninth = thirty_eighth + 39;
    let fortieth = thirty_ninth + 40;
    fortieth
}
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
        .find(|finding| finding.rule_id == "MOD-R007")
        .unwrap_or_else(|| panic!("expected MOD-R007 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("navigational table of contents"),
        "expected root-toc title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_clear_folder_root_toc() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub(crate) use parser::Parser;
pub(crate) use runtime::Runtime;
pub(crate) use service::Service;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R007"),
        "expected no MOD-R007 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_noisy_root_facade_exports() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub use self::{
    parser::{ParseError, ParseMode, ParsePlan},
    runtime::{Runtime, RuntimeConfig, RuntimeHandle},
    service::{Service, ServiceRequest, ServiceResponse},
};
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
        .find(|finding| finding.rule_id == "MOD-R008")
        .unwrap_or_else(|| panic!("expected MOD-R008 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("export surface"),
        "expected root-facade title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_curated_root_facade_exports() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub use self::parser::Parser;
pub use self::runtime::Runtime;
pub use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub struct Parser;");
    write_rust_file(&src_root, "feature/runtime.rs", "pub struct Runtime;");
    write_rust_file(&src_root, "feature/service.rs", "pub struct Service;");

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R008"),
        "expected no MOD-R008 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_multi_hop_relative_imports() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/worker.rs",
        r"
use super::super::shared::SharedState;

pub(crate) fn run(state: SharedState) -> usize {
    state.value()
}
",
    );
    write_rust_file(
        &src_root,
        "shared.rs",
        r"
pub(crate) struct SharedState(usize);

impl SharedState {
    pub(crate) fn value(&self) -> usize {
        self.0
    }
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R009")
        .unwrap_or_else(|| panic!("expected MOD-R009 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("Prefer `crate::`"),
        "expected relative-import title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_crate_qualified_imports() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature/worker.rs",
        r"
use crate::shared::SharedState;

pub(crate) fn run(state: SharedState) -> usize {
    state.value()
}
",
    );
    write_rust_file(
        &src_root,
        "shared.rs",
        r"
pub(crate) struct SharedState(usize);

impl SharedState {
    pub(crate) fn value(&self) -> usize {
        self.0
    }
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R009"),
        "expected no MOD-R009 finding, got {findings:#?}"
    );
}

#[test]
fn modularity_pack_flags_public_alias_reexport_in_root_facade() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub use self::parser::Parser as FeatureParser;
pub use self::runtime::Runtime;
pub use self::service::Service;
",
    );
    write_rust_file(&src_root, "feature/parser.rs", "pub struct Parser;");
    write_rust_file(&src_root, "feature/runtime.rs", "pub struct Runtime;");
    write_rust_file(&src_root, "feature/service.rs", "pub struct Service;");

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R010")
        .unwrap_or_else(|| panic!("expected MOD-R010 finding, got {findings:#?}"));
    assert!(
        finding.title.contains("alias re-exports"),
        "expected root-alias title, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_restricted_alias_reexport_in_root_facade() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "feature.rs",
        r"
mod parser;
mod runtime;
mod service;

pub(crate) use self::parser::Parser as InternalParser;
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
        findings.iter().all(|finding| finding.rule_id != "MOD-R010"),
        "expected no MOD-R010 finding, got {findings:#?}"
    );
}
