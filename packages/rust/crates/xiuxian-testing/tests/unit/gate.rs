use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use super::*;

fn create_temp_crate() -> tempfile::TempDir {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    if let Err(error) = fs::create_dir_all(temp.path().join("src")) {
        panic!("src dir should be created: {error}");
    }
    let manifest = "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    if let Err(error) = fs::write(temp.path().join("Cargo.toml"), manifest) {
        panic!("Cargo.toml should be written: {error}");
    }
    temp
}

fn write_fixture_file(crate_root: &Path, relative_path: &str, content: &str) {
    let path = crate_root.join(relative_path);
    let Some(parent) = path.parent() else {
        panic!("fixture path should have parent: {path:?}");
    };
    if let Err(error) = fs::create_dir_all(parent) {
        panic!("fixture directories should be created: {error}");
    }
    if let Err(error) = fs::write(path, content) {
        panic!("fixture file should be written: {error}");
    }
}

#[test]
fn collect_crate_modularity_findings_reports_bloated_bin_source() {
    let temp = create_temp_crate();
    let mut content = String::new();
    for idx in 0..24 {
        let _ = writeln!(
            content,
            "fn helper_{idx}() -> usize {{\n    let base = {idx};\n    let adjusted = base + 1;\n    adjusted + 1\n}}\n"
        );
    }
    for idx in 0..18 {
        let _ = writeln!(
            content,
            "struct State{idx} {{\n    value: usize,\n}}\n\nimpl State{idx} {{\n    fn value(&self) -> usize {{\n        self.value\n    }}\n}}\n"
        );
    }
    write_fixture_file(temp.path(), "src/bin/qianji.rs", &content);

    let findings = collect_crate_modularity_findings(temp.path())
        .unwrap_or_else(|error| panic!("modularity collection should succeed: {error}"));
    assert!(
        findings.iter().any(|finding| {
            finding.rule_id == "MOD-R006"
                && finding
                    .evidence
                    .iter()
                    .find_map(|evidence| evidence.path.as_ref())
                    .is_some_and(|path| path.ends_with("src/bin/qianji.rs"))
        }),
        "expected MOD-R006 for bloated bin source, got {findings:#?}"
    );
}

#[test]
fn assert_crate_modularity_gate_blocks_warning_findings_by_default() {
    let temp = create_temp_crate();
    let mut content = String::new();
    for idx in 0..24 {
        let _ = writeln!(
            content,
            "fn helper_{idx}() -> usize {{\n    let base = {idx};\n    let adjusted = base + 1;\n    adjusted + 1\n}}\n"
        );
    }
    for idx in 0..18 {
        let _ = writeln!(
            content,
            "struct State{idx} {{\n    value: usize,\n}}\n\nimpl State{idx} {{\n    fn value(&self) -> usize {{\n        self.value\n    }}\n}}\n"
        );
    }
    write_fixture_file(temp.path(), "src/bin/qianji.rs", &content);

    let result = std::panic::catch_unwind(|| {
        assert_crate_modularity_gate(temp.path());
    });
    assert!(
        result.is_err(),
        "expected MOD-R006 warning to block by default"
    );
}

#[test]
fn format_crate_modularity_gate_report_includes_title_why_and_fix() {
    let mut finding = crate::ContractFinding::new(
        "MOD-R001",
        "modularity",
        crate::FindingSeverity::Error,
        crate::FindingMode::Deterministic,
        "mod.rs should remain interface-only",
        "src/feature/mod.rs exposes a visible module declaration.",
    );
    finding.why_it_matters =
        "Visible child modules in mod.rs hide the intended curated seam.".to_string();
    finding.remediation =
        "Replace the visible child with a private #[path = \"../feature_impl.rs\"] mod feature_impl; mount and a selective pub(crate) use."
            .to_string();
    finding.evidence.push(crate::FindingEvidence {
        kind: crate::EvidenceKind::SourceSpan,
        path: Some(Path::new("src/feature/mod.rs").to_path_buf()),
        locator: Some("line 3".to_string()),
        message: "pub mod feature_impl;".to_string(),
    });

    let report = format_crate_modularity_gate_report(&[&finding]);
    assert!(
        report.contains("[MOD-R001][Error] mod.rs should remain interface-only"),
        "expected title and severity in report, got {report}"
    );
    assert!(
        report.contains("summary: src/feature/mod.rs exposes a visible module declaration."),
        "expected summary line in report, got {report}"
    );
    assert!(
        report.contains("why: Visible child modules in mod.rs hide the intended curated seam."),
        "expected why line in report, got {report}"
    );
    assert!(
        report.contains("fix: Replace the visible child with a private #[path = \"../feature_impl.rs\"] mod feature_impl; mount and a selective pub(crate) use."),
        "expected remediation line in report, got {report}"
    );
}
