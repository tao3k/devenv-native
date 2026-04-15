use super::*;
use std::fmt::Write as _;

#[test]
fn modularity_pack_flags_public_result_without_errors_doc() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "api.rs",
        r"
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(findings.iter().any(|finding| finding.rule_id == "MOD-R003"));
}

#[test]
fn modularity_pack_flags_overly_broad_visibility_in_internal_module() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "internal/state.rs",
        r"
pub struct InternalState {
    value: usize,
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(findings.iter().any(|finding| finding.rule_id == "MOD-R002"));
}

#[test]
fn modularity_pack_flags_bloated_multi_responsibility_file() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");

    let mut content = String::from(
        "pub struct Planner {\n    value: usize,\n}\n\npub struct RuntimeState {\n    active: bool,\n}\n\npub enum Mode {\n    Fast,\n    Safe,\n}\n\npub const DEFAULT_LIMIT: usize = 32;\n\n",
    );
    for idx in 0..24 {
        write!(
            content,
            "pub fn helper_{idx}(input: usize) -> usize {{\n    let base = input + {idx};\n    let staged = base + DEFAULT_LIMIT;\n    let guarded = staged.saturating_add({idx});\n    guarded + 1\n}}\n\n"
        )
        .unwrap_or_else(|error| panic!("should append helper fixture body: {error}"));
    }
    for idx in 0..18 {
        write!(
            content,
            "impl Planner {{\n    pub fn stage_{idx}(&self) -> usize {{\n        let local = self.value + {idx};\n        let bounded = local + DEFAULT_LIMIT;\n        bounded.saturating_sub({idx})\n    }}\n}}\n\n"
        )
        .unwrap_or_else(|error| panic!("should append impl fixture body: {error}"));
    }

    write_rust_file(&src_root, "feature.rs", &content);

    let findings = evaluate_fixture("demo", &temp_dir);
    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "MOD-R006")
        .unwrap_or_else(|| panic!("expected MOD-R006 finding, got {findings:#?}"));
    assert!(
        finding
            .title
            .contains("appears too large for one ownership seam"),
        "expected file-bloat summary, got {finding:#?}"
    );
}

#[test]
fn modularity_pack_accepts_compact_single_responsibility_file() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let src_root = crate_src_root(&temp_dir, "demo");
    write_rust_file(
        &src_root,
        "service.rs",
        r"
pub struct Service {
    value: usize,
}

impl Service {
    pub fn new(value: usize) -> Self {
        Self { value }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}
",
    );

    let findings = evaluate_fixture("demo", &temp_dir);
    assert!(
        findings.iter().all(|finding| finding.rule_id != "MOD-R006"),
        "expected no MOD-R006 finding, got {findings:#?}"
    );
}
