//! Relative visibility regression gate for the Wendao crate.

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn enforce_no_new_relative_ancestor_visibility_gate() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let findings = collect_relative_ancestor_visibility_findings(crate_root);
    let blocking_findings = findings
        .iter()
        .filter(|finding| !is_legacy_relative_visibility_finding(finding))
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_relative_visibility_gate_report(&blocking_findings)
    );
}

const LEGACY_RELATIVE_ANCESTOR_VISIBILITY_BASELINE: &[&str] = &[];

#[derive(Debug)]
struct RelativeVisibilityFinding {
    relative_path: String,
    line_number: usize,
    declaration: String,
}

fn collect_relative_ancestor_visibility_findings(
    crate_root: &Path,
) -> Vec<RelativeVisibilityFinding> {
    let source_root = crate_root.join("src");
    let mut files = Vec::new();
    collect_rust_source_files(source_root.as_path(), &mut files)
        .unwrap_or_else(|error| panic!("failed to collect Rust source files: {error}"));
    let mut findings = Vec::new();
    for path in files {
        let content = fs::read_to_string(path.as_path())
            .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()));
        let relative_path = path.strip_prefix(crate_root).map_or_else(
            |_| path.display().to_string(),
            |relative| relative.display().to_string(),
        );
        findings.extend(
            content
                .lines()
                .enumerate()
                .filter_map(|(line_index, line)| {
                    let declaration = line.trim();
                    declaration
                        .contains("pub(in super::")
                        .then(|| RelativeVisibilityFinding {
                            relative_path: relative_path.clone(),
                            line_number: line_index + 1,
                            declaration: declaration.to_string(),
                        })
                }),
        );
    }
    findings
}

fn collect_rust_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(path.as_path(), files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn is_legacy_relative_visibility_finding(finding: &RelativeVisibilityFinding) -> bool {
    let key = relative_visibility_key(finding);
    LEGACY_RELATIVE_ANCESTOR_VISIBILITY_BASELINE
        .iter()
        .any(|baseline| key == *baseline)
}

fn relative_visibility_key(finding: &RelativeVisibilityFinding) -> String {
    format!("{}::{}", finding.relative_path, finding.declaration)
}

fn format_relative_visibility_gate_report(findings: &[&RelativeVisibilityFinding]) -> String {
    let mut output = String::from(
        "relative ancestor visibility gate failed with new `pub(in super::...)` declarations:\n",
    );
    for finding in findings {
        let _ = writeln!(
            output,
            "- {}:{} :: {}",
            finding.relative_path, finding.line_number, finding.declaration
        );
    }
    output
}
