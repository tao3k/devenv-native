use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use orgize::Org;
use xiuxian_qianji_bpmn_engine::{BpmnSourceFile, lint_bpmn_source};

use crate::contracts::FlowhubStructureContract;
use crate::error::QianjiError;
use crate::flowhub::discover::FlowhubDiscoveredModule;

use super::contract::expanded_required_entries;
use super::model::FlowhubDiagnostic;

pub(super) fn validate_org_bpmn_source_pairs(
    module: &FlowhubDiscoveredModule,
    contract: &FlowhubStructureContract,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let required_entries = expanded_required_entries(contract);
    for pair in source_pairs_from_required_entries(&required_entries).into_values() {
        validate_source_pair(module, &pair, diagnostics)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct SourcePair {
    org: Option<String>,
    bpmn: Option<String>,
}

fn source_pairs_from_required_entries(required_entries: &[String]) -> BTreeMap<String, SourcePair> {
    let mut pairs = BTreeMap::new();
    let bpmn_keys: BTreeSet<_> = required_entries
        .iter()
        .filter_map(|entry| {
            let (key, extension) = source_pair_key(entry)?;
            (extension == "bpmn").then_some(key)
        })
        .collect();

    for entry in required_entries {
        let Some((key, extension)) = source_pair_key(entry) else {
            continue;
        };
        if extension == "org" && is_policy_org_entry(entry) && !bpmn_keys.contains(&key) {
            continue;
        }
        let pair: &mut SourcePair = pairs.entry(key).or_default();
        match extension.as_str() {
            "org" => pair.org = Some(entry.clone()),
            "bpmn" => pair.bpmn = Some(entry.clone()),
            _ => {}
        }
    }
    pairs
}

fn is_policy_org_entry(entry: &str) -> bool {
    Path::new(entry)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "POLICY.org" || name.ends_with("_POLICY.org"))
}

fn source_pair_key(entry: &str) -> Option<(String, String)> {
    let path = Path::new(entry);
    let extension = path.extension()?.to_str()?.to_string();
    if extension != "org" && extension != "bpmn" {
        return None;
    }

    let mut key = PathBuf::from(entry);
    key.set_extension("");
    Some((key.to_string_lossy().into_owned(), extension))
}

fn validate_source_pair(
    module: &FlowhubDiscoveredModule,
    pair: &SourcePair,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    match (&pair.org, &pair.bpmn) {
        (Some(org_file), Some(bpmn_file)) => {
            validate_org_source(module, org_file, bpmn_file, diagnostics);
            validate_bpmn_source(module, bpmn_file, diagnostics)?;
        }
        (Some(org_file), None) => diagnostics.push(incomplete_pair_diagnostic(
            module,
            org_file,
            "the module declares an Org scenario source without the matching BPMN source",
            "add the sibling `.bpmn` source to `contract.required` and the module directory",
        )),
        (None, Some(bpmn_file)) => diagnostics.push(incomplete_pair_diagnostic(
            module,
            bpmn_file,
            "the module declares a BPMN scenario source without the matching Org source",
            "add the sibling `.org` source to `contract.required` and the module directory",
        )),
        (None, None) => {}
    }
    Ok(())
}

fn incomplete_pair_diagnostic(
    module: &FlowhubDiscoveredModule,
    declared_file: &str,
    problem: &str,
    fix: &str,
) -> FlowhubDiagnostic {
    FlowhubDiagnostic {
        title: "Incomplete Flowhub Org+BPMN scenario source pair".to_string(),
        location: module.module_dir.join(declared_file),
        problem: problem.to_string(),
        why_it_blocks: "Qianji cannot bind durable scenario intent to executable topology"
            .to_string(),
        fix: fix.to_string(),
    }
}

fn validate_org_source(
    module: &FlowhubDiscoveredModule,
    org_file: &str,
    bpmn_file: &str,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) {
    let path = module.module_dir.join(org_file);
    if !path.is_file() {
        return;
    }

    let Ok(source) = fs::read_to_string(&path) else {
        diagnostics.push(FlowhubDiagnostic {
            title: "Unreadable Flowhub Org scenario source".to_string(),
            location: path,
            problem: "failed to read the Org scenario source".to_string(),
            why_it_blocks: "Qianji cannot inspect the Org-owned scenario semantics".to_string(),
            fix: "repair the Org scenario source path and permissions".to_string(),
        });
        return;
    };

    if let Some(problem) = validate_org_syntax(&path, &source) {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid Flowhub Org scenario source".to_string(),
            location: path.clone(),
            problem,
            why_it_blocks: "Qianji cannot trust the Org-owned scenario semantics".to_string(),
            fix: "repair the Org scenario source so it has a parseable review heading".to_string(),
        });
    }

    validate_org_bpmn_property(&path, &source, bpmn_file, diagnostics);
    validate_org_mermaid_babel(&path, &source, diagnostics);
}

fn validate_org_syntax(path: &Path, source: &str) -> Option<String> {
    let _document = Org::parse(source).document();
    if source.trim().is_empty() {
        return Some(format!("Org scenario source `{}` is empty", path.display()));
    }
    if !source
        .lines()
        .any(|line| line.trim_start().starts_with('*'))
    {
        return Some(format!(
            "Org scenario source `{}` has no review heading",
            path.display()
        ));
    }
    None
}

fn validate_org_bpmn_property(
    path: &Path,
    source: &str,
    bpmn_file: &str,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) {
    let bpmn_source = Path::new(bpmn_file)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(bpmn_file);
    let expected = format!(":BPMN_SOURCE: {bpmn_source}");
    if source.contains(&expected) {
        return;
    }

    diagnostics.push(FlowhubDiagnostic {
        title: "Missing Flowhub Org BPMN source binding".to_string(),
        location: path.to_path_buf(),
        problem: format!("Org scenario source must declare `{expected}`"),
        why_it_blocks: "Qianji cannot deterministically bind the Org scenario to its BPMN topology"
            .to_string(),
        fix: "add the exact `:BPMN_SOURCE:` property to the scenario property drawer".to_string(),
    });
}

fn validate_org_mermaid_babel(path: &Path, source: &str, diagnostics: &mut Vec<FlowhubDiagnostic>) {
    let lowered = source.to_ascii_lowercase();
    if lowered.contains("#+begin_src mermaid") && lowered.contains("#+end_src") {
        return;
    }

    diagnostics.push(FlowhubDiagnostic {
        title: "Missing Flowhub Org Mermaid Babel block".to_string(),
        location: path.to_path_buf(),
        problem: "Org scenario source must embed the Mermaid read model in a Babel block"
            .to_string(),
        why_it_blocks: "Qianji cannot keep the human-readable graph view co-located with the scenario authority"
            .to_string(),
        fix: "add a `#+begin_src mermaid` block with the scenario flowchart to the Org source"
            .to_string(),
    });
}

fn validate_bpmn_source(
    module: &FlowhubDiscoveredModule,
    bpmn_file: &str,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let path = module.module_dir.join(bpmn_file);
    if !path.is_file() {
        return Ok(());
    }

    let source = fs::read_to_string(&path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub BPMN source `{}`: {error}",
            path.display()
        ))
    })?;
    let report = lint_bpmn_source(&BpmnSourceFile::new(path.display().to_string(), source));
    if report.ok {
        return Ok(());
    }

    for issue in report.issues {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid Flowhub BPMN scenario source".to_string(),
            location: PathBuf::from(&report.source_id),
            problem: format!("{}: {}", issue.code, issue.summary),
            why_it_blocks: "Qianji cannot trust the BPMN-owned executable topology".to_string(),
            fix: issue.llm_fix_prompt,
        });
    }

    Ok(())
}
