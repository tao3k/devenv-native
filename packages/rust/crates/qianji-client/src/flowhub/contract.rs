//! Lightweight Flowhub contract validation for the client CLI.

use std::fs;
use std::path::{Path, PathBuf};

use qianji_bpmn_engine::{BpmnSourceFile, lint_bpmn_source};
use xiuxian_wendao_parsers::{OrgizeLintOutputFormat, OrgizeLintRequest, lint_org_files};

use crate::QianjiClientError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubSourcePair {
    pub(crate) scenario_id: String,
    pub(crate) org_source: PathBuf,
    pub(crate) bpmn_source: PathBuf,
    pub(crate) bpmn_source_name: String,
    pub(crate) bpmn_process_id: String,
}

pub(crate) fn validate_flowhub_source_pair_contract(
    flowhub_root: &Path,
    scenario: &str,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let Some(source_pair) = resolve_flowhub_source_pair(flowhub_root, scenario, diagnostics)?
    else {
        return Ok(false);
    };

    let mut passed = true;
    passed &= validate_org_source(&source_pair, diagnostics)?;
    passed &= validate_bpmn_source(&source_pair, diagnostics)?;
    Ok(passed)
}

pub(crate) fn list_flowhub_source_pairs(
    flowhub_root: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<Vec<FlowhubSourcePair>, QianjiClientError> {
    let mut source_pairs = collect_flowhub_source_pairs(flowhub_root, diagnostics)?;
    if !validate_unique_scenario_ids(&source_pairs, diagnostics) {
        return Ok(Vec::new());
    }
    source_pairs.sort_by(|left, right| left.scenario_id.cmp(&right.scenario_id));
    Ok(source_pairs)
}

pub(crate) fn resolve_flowhub_source_pair(
    flowhub_root: &Path,
    scenario: &str,
    diagnostics: &mut Vec<String>,
) -> Result<Option<FlowhubSourcePair>, QianjiClientError> {
    let source_pairs = collect_flowhub_source_pairs(flowhub_root, diagnostics)?;
    if !validate_unique_scenario_ids(&source_pairs, diagnostics) {
        return Ok(None);
    }
    let source_pair = source_pairs
        .into_iter()
        .find(|source_pair| source_pair.scenario_id == scenario);
    if source_pair.is_some() {
        return Ok(source_pair);
    }

    diagnostics.push(format!(
        "Flowhub root `{}` has no Org source with FLOWHUB_SCENARIO_ID `{scenario}`",
        flowhub_root.display()
    ));
    Ok(None)
}

fn collect_flowhub_source_pairs(
    flowhub_root: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<Vec<FlowhubSourcePair>, QianjiClientError> {
    let mut org_sources = Vec::new();
    collect_org_sources(flowhub_root, 0, &mut org_sources)?;
    org_sources
        .into_iter()
        .map(|org_source| parse_source_pair_from_org_path(flowhub_root, &org_source, diagnostics))
        .collect::<Result<Vec<_>, _>>()
        .map(|source_pairs| source_pairs.into_iter().flatten().collect())
}

fn validate_unique_scenario_ids(
    source_pairs: &[FlowhubSourcePair],
    diagnostics: &mut Vec<String>,
) -> bool {
    let mut sorted = source_pairs.to_vec();
    sorted.sort_by(|left, right| {
        left.scenario_id
            .cmp(&right.scenario_id)
            .then_with(|| left.org_source.cmp(&right.org_source))
    });
    let duplicates = sorted
        .windows(2)
        .filter(|window| window[0].scenario_id == window[1].scenario_id)
        .map(|window| {
            format!(
                "duplicate Flowhub scenario id `{}` in `{}` and `{}`",
                window[0].scenario_id,
                window[0].org_source.display(),
                window[1].org_source.display()
            )
        })
        .collect::<Vec<_>>();
    diagnostics.extend(duplicates);
    sorted
        .windows(2)
        .all(|window| window[0].scenario_id != window[1].scenario_id)
}

fn parse_source_pair_from_org_path(
    flowhub_root: &Path,
    org_source: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<Option<FlowhubSourcePair>, QianjiClientError> {
    let source = read_to_string(org_source, "Flowhub Org source")?;
    Ok(parse_source_pair_from_org(
        flowhub_root,
        org_source,
        &source,
        diagnostics,
    ))
}

fn parse_source_pair_from_org(
    flowhub_root: &Path,
    org_source: &Path,
    source: &str,
    diagnostics: &mut Vec<String>,
) -> Option<FlowhubSourcePair> {
    let properties = parse_org_properties(source);
    let scenario_id = property_value(&properties, "FLOWHUB_SCENARIO_ID")?;
    let Some(bpmn_source_name) = property_value(&properties, "BPMN_SOURCE") else {
        diagnostics.push(format!(
            "Flowhub Org source `{}` for scenario `{scenario_id}` is missing BPMN_SOURCE",
            org_source.display()
        ));
        return None;
    };
    let Some(bpmn_process_id) = property_value(&properties, "BPMN_PROCESS_ID") else {
        diagnostics.push(format!(
            "Flowhub Org source `{}` for scenario `{scenario_id}` is missing BPMN_PROCESS_ID",
            org_source.display()
        ));
        return None;
    };
    let bpmn_source = org_source
        .parent()
        .unwrap_or(flowhub_root)
        .join(bpmn_source_name);
    if !bpmn_source.is_file() {
        diagnostics.push(format!(
            "missing Flowhub BPMN source `{}` declared by `{}`",
            bpmn_source.display(),
            org_source.display()
        ));
        return None;
    }
    Some(FlowhubSourcePair {
        scenario_id: scenario_id.to_string(),
        org_source: org_source.to_path_buf(),
        bpmn_source,
        bpmn_source_name: bpmn_source_name.to_string(),
        bpmn_process_id: bpmn_process_id.to_string(),
    })
}

fn collect_org_sources(
    dir: &Path,
    depth: usize,
    org_sources: &mut Vec<PathBuf>,
) -> Result<(), QianjiClientError> {
    if depth > 4 || !dir.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read Flowhub directory `{}`: {error}",
            dir.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read Flowhub directory entry under `{}`: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_org_sources(&path, depth + 1, org_sources)?;
        } else if path.extension().is_some_and(|extension| extension == "org") {
            org_sources.push(path);
        }
    }
    org_sources.sort();
    Ok(())
}

fn validate_org_source(
    source_pair: &FlowhubSourcePair,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let source = read_to_string(&source_pair.org_source, "Flowhub Org source")?;
    let properties = parse_org_properties(&source);
    let mut passed = true;
    if property_value(&properties, "FLOWHUB_SCENARIO_ID") != Some(source_pair.scenario_id.as_str())
    {
        diagnostics.push(format!(
            "Flowhub Org source `{}` does not bind scenario `{}`",
            source_pair.org_source.display(),
            source_pair.scenario_id
        ));
        passed = false;
    }
    if property_value(&properties, "CANONICAL_SOURCE") != Some("org+bpmn") {
        diagnostics.push(format!(
            "Flowhub Org source `{}` does not declare CANONICAL_SOURCE `org+bpmn`",
            source_pair.org_source.display()
        ));
        passed = false;
    }
    if property_value(&properties, "BPMN_SOURCE") != Some(source_pair.bpmn_source_name.as_str()) {
        diagnostics.push(format!(
            "Flowhub Org source `{}` does not bind BPMN source `{}`",
            source_pair.org_source.display(),
            source_pair.bpmn_source_name
        ));
        passed = false;
    }
    if property_value(&properties, "BPMN_PROCESS_ID") != Some(source_pair.bpmn_process_id.as_str())
    {
        diagnostics.push(format!(
            "Flowhub Org source `{}` does not bind BPMN process `{}`",
            source_pair.org_source.display(),
            source_pair.bpmn_process_id
        ));
        passed = false;
    }
    if !source.contains("#+begin_src mermaid") {
        diagnostics.push(format!(
            "Flowhub Org source `{}` does not contain a Mermaid Babel block",
            source_pair.org_source.display()
        ));
        passed = false;
    }

    let request = OrgizeLintRequest {
        paths: vec![source_pair.org_source.clone()],
        output_format: OrgizeLintOutputFormat::Compact,
        priority_highest: None,
        priority_lowest: None,
        priority_default: None,
    };
    let report = lint_org_files(&request).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to lint Flowhub Org source `{}`: {error}",
            source_pair.org_source.display()
        ))
    })?;
    if !report.is_clean() {
        diagnostics.push(report.render(OrgizeLintOutputFormat::Compact));
        passed = false;
    }
    Ok(passed)
}

fn validate_bpmn_source(
    source_pair: &FlowhubSourcePair,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let source = read_to_string(&source_pair.bpmn_source, "Flowhub BPMN source")?;
    let process_marker = format!("id=\"{}\"", source_pair.bpmn_process_id);
    if !source.contains(&process_marker) {
        diagnostics.push(format!(
            "Flowhub BPMN source `{}` does not contain process id `{}`",
            source_pair.bpmn_source.display(),
            source_pair.bpmn_process_id
        ));
        return Ok(false);
    }
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        source_pair.bpmn_source.display().to_string(),
        source,
    ));
    if report.ok {
        return Ok(true);
    }

    for issue in report.issues {
        diagnostics.push(format!(
            "Flowhub BPMN source `{}` failed {}: {}",
            report.source_id, issue.code, issue.summary
        ));
    }
    Ok(false)
}

fn read_to_string(path: &Path, label: &str) -> Result<String, QianjiClientError> {
    fs::read_to_string(path).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read {label} `{}`: {error}",
            path.display()
        ))
    })
}

pub(crate) fn parse_org_properties(source: &str) -> Vec<(String, String)> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let remainder = trimmed.strip_prefix(':')?;
            let (key, value) = remainder.split_once(':')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub(crate) fn property_value<'a>(properties: &'a [(String, String)], key: &str) -> Option<&'a str> {
    properties
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}
