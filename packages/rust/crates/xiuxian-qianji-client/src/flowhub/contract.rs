//! Lightweight Flowhub contract validation for the client CLI.

use std::fs;
use std::path::{Path, PathBuf};

use orgize::{Org, ast::OrgElementSelector};
use serde::Deserialize;
use xiuxian_qianji_bpmn_engine::{BpmnSourceFile, lint_bpmn_source};
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

#[derive(Debug, Deserialize)]
struct FlowhubManifest {
    module: Option<FlowhubManifestModule>,
    contract: Option<FlowhubManifestContract>,
}

#[derive(Debug, Deserialize)]
struct FlowhubManifestModule {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct FlowhubManifestContract {
    #[serde(default)]
    required: Vec<String>,
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

pub(crate) fn validate_flowhub_module_policy_entries(
    flowhub_root: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let mut manifest_paths = Vec::new();
    collect_manifest_paths(flowhub_root, 0, &mut manifest_paths)?;
    let mut passed = true;
    for manifest_path in manifest_paths {
        let source = read_to_string(&manifest_path, "Flowhub module manifest")?;
        let manifest = toml::from_str::<FlowhubManifest>(&source).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to parse Flowhub module manifest `{}`: {error}",
                manifest_path.display()
            ))
        })?;
        let Some(module) = manifest.module else {
            continue;
        };
        let module_dir = manifest_path.parent().unwrap_or(flowhub_root);
        let contract = manifest.contract.unwrap_or_default();
        passed &= validate_module_policy_entry(
            flowhub_root,
            module_dir,
            &manifest_path,
            &module.name,
            &contract,
            diagnostics,
        )?;
    }
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

fn validate_module_policy_entry(
    flowhub_root: &Path,
    module_dir: &Path,
    manifest_path: &Path,
    module_name: &str,
    contract: &FlowhubManifestContract,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let policy_name = policy_filename_for_module_name(module_name);
    let policy_path = module_dir.join(&policy_name);
    let mut passed = true;
    passed &= validate_module_required_surfaces(
        flowhub_root,
        module_dir,
        manifest_path,
        &contract.required,
        diagnostics,
    )?;
    if !contract
        .required
        .iter()
        .any(|required| required == &policy_name)
    {
        diagnostics.push(format!(
            "Flowhub module manifest `{}` must list required policy entry `{policy_name}`",
            manifest_path.display()
        ));
        passed = false;
    }
    if !has_exact_relative_file(module_dir, Path::new(&policy_name))? {
        diagnostics.push(format!(
            "Flowhub module `{}` is missing required policy entry `{}`",
            module_path_label(flowhub_root, module_dir),
            policy_path.display()
        ));
        return Ok(false);
    }

    passed &= validate_policy_org_source(&policy_path, module_name, diagnostics)?;
    Ok(passed)
}

fn validate_module_required_surfaces(
    flowhub_root: &Path,
    module_dir: &Path,
    manifest_path: &Path,
    required_surfaces: &[String],
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let mut passed = true;
    for required_surface in required_surfaces {
        let Some(relative_path) =
            parse_required_surface_path(manifest_path, required_surface, diagnostics)
        else {
            passed = false;
            continue;
        };
        let surface_path = module_dir.join(relative_path);
        if !has_exact_relative_file(module_dir, relative_path)? {
            diagnostics.push(format!(
                "Flowhub module manifest `{}` lists missing required surface `{}` under module `{}`",
                manifest_path.display(),
                required_surface,
                module_path_label(flowhub_root, module_dir)
            ));
            passed = false;
            continue;
        }
        passed &= validate_required_surface_content(
            manifest_path,
            required_surface,
            &surface_path,
            diagnostics,
        )?;
    }
    Ok(passed)
}

fn parse_required_surface_path<'a>(
    manifest_path: &Path,
    required_surface: &'a str,
    diagnostics: &mut Vec<String>,
) -> Option<&'a Path> {
    let relative_path = Path::new(required_surface);
    let has_parent_component = relative_path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir));
    if required_surface.trim().is_empty() || relative_path.is_absolute() || has_parent_component {
        diagnostics.push(format!(
            "Flowhub module manifest `{}` has invalid required surface `{required_surface}`; surfaces must be non-empty relative paths inside the module",
            manifest_path.display()
        ));
        return None;
    }
    Some(relative_path)
}

fn validate_required_surface_content(
    manifest_path: &Path,
    required_surface: &str,
    surface_path: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    match surface_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("org") => validate_required_org_surface(
            manifest_path,
            required_surface,
            surface_path,
            diagnostics,
        ),
        Some("bpmn") => validate_required_bpmn_surface(
            manifest_path,
            required_surface,
            surface_path,
            diagnostics,
        ),
        _ => Ok(true),
    }
}

fn validate_required_org_surface(
    manifest_path: &Path,
    required_surface: &str,
    surface_path: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let request = OrgizeLintRequest {
        paths: vec![surface_path.to_path_buf()],
        output_format: OrgizeLintOutputFormat::Compact,
        priority_highest: None,
        priority_lowest: None,
        priority_default: None,
        fix: false,
    };
    let report = lint_org_files(&request).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to lint required Flowhub Org surface `{}` from manifest `{}`: {error}",
            required_surface,
            manifest_path.display()
        ))
    })?;
    if report.is_clean() {
        return Ok(true);
    }
    diagnostics.push(report.render(OrgizeLintOutputFormat::Compact));
    Ok(false)
}

fn validate_required_bpmn_surface(
    manifest_path: &Path,
    required_surface: &str,
    surface_path: &Path,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let source = read_to_string(surface_path, "required Flowhub BPMN surface")?;
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        format!(
            "{} required surface `{required_surface}`",
            manifest_path.display()
        ),
        source,
    ));
    if report.ok {
        return Ok(true);
    }
    for issue in report.issues {
        diagnostics.push(format!(
            "Flowhub module manifest `{}` required surface `{}` failed {}: {}",
            manifest_path.display(),
            required_surface,
            issue.code,
            issue.summary
        ));
    }
    Ok(false)
}

fn validate_policy_org_source(
    policy_path: &Path,
    module_name: &str,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let source = read_to_string(policy_path, "Flowhub policy Org source")?;
    let properties = parse_org_properties(&source);
    let mut passed = true;
    if property_value(&properties, "FLOWHUB_POLICY_ENTRY").is_none() {
        diagnostics.push(format!(
            "Flowhub policy entry `{}` is missing FLOWHUB_POLICY_ENTRY",
            policy_path.display()
        ));
        passed = false;
    }
    let expected_mode = policy_mode_for_module_name(module_name);
    if property_value(&properties, "FLOWHUB_POLICY_MODE") != Some(expected_mode.as_str()) {
        diagnostics.push(format!(
            "Flowhub policy entry `{}` must declare FLOWHUB_POLICY_MODE `{expected_mode}`",
            policy_path.display()
        ));
        passed = false;
    }
    passed &=
        validate_policy_contract_graph_selector(policy_path, &source, &properties, diagnostics);

    let request = OrgizeLintRequest {
        paths: vec![policy_path.to_path_buf()],
        output_format: OrgizeLintOutputFormat::Compact,
        priority_highest: None,
        priority_lowest: None,
        priority_default: None,
        fix: false,
    };
    let report = lint_org_files(&request).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to lint Flowhub policy entry `{}`: {error}",
            policy_path.display()
        ))
    })?;
    if !report.is_clean() {
        diagnostics.push(report.render(OrgizeLintOutputFormat::Compact));
        passed = false;
    }
    Ok(passed)
}

fn validate_policy_contract_graph_selector(
    policy_path: &Path,
    source: &str,
    properties: &[(String, String)],
    diagnostics: &mut Vec<String>,
) -> bool {
    let Some(selector_source) = property_value(properties, "FLOWHUB_CONTRACT_GRAPH") else {
        return true;
    };
    let selector = match OrgElementSelector::parse_plist(selector_source) {
        Ok(selector) => selector,
        Err(error) => {
            diagnostics.push(format!(
                "Flowhub policy entry `{}` has invalid FLOWHUB_CONTRACT_GRAPH selector: {error}",
                policy_path.display()
            ));
            return false;
        }
    };
    let document = Org::parse(source).document();
    let matches = document.select_org_elements(&selector);
    match matches.len() {
        1 => true,
        0 => {
            diagnostics.push(format!(
                "Flowhub policy entry `{}` FLOWHUB_CONTRACT_GRAPH selector matched no Org element",
                policy_path.display()
            ));
            false
        }
        count => {
            diagnostics.push(format!(
                "Flowhub policy entry `{}` FLOWHUB_CONTRACT_GRAPH selector matched {count} Org elements; expected exactly 1",
                policy_path.display()
            ));
            false
        }
    }
}

fn policy_filename_for_module_name(module_name: &str) -> String {
    format!("{}_POLICY.org", policy_mode_for_module_name(module_name))
}

fn policy_mode_for_module_name(module_name: &str) -> String {
    let mut mode = String::new();
    let mut previous_was_separator = false;
    for character in module_name.chars() {
        if character.is_ascii_alphanumeric() {
            mode.push(character.to_ascii_uppercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !mode.is_empty() {
            mode.push('_');
            previous_was_separator = true;
        }
    }
    while mode.ends_with('_') {
        mode.pop();
    }
    if mode.is_empty() {
        "MODULE".to_string()
    } else {
        mode
    }
}

fn module_path_label(flowhub_root: &Path, module_dir: &Path) -> String {
    module_dir
        .strip_prefix(flowhub_root)
        .unwrap_or(module_dir)
        .display()
        .to_string()
}

fn has_exact_relative_file(dir: &Path, relative_path: &Path) -> Result<bool, QianjiClientError> {
    let mut current_dir = dir.to_path_buf();
    let mut components = relative_path.components().peekable();
    while let Some(component) = components.next() {
        let std::path::Component::Normal(component_name) = component else {
            return Ok(false);
        };
        let is_last = components.peek().is_none();
        let Some(next_path) = exact_child_path(&current_dir, component_name)? else {
            return Ok(false);
        };
        if is_last {
            return Ok(next_path.is_file());
        }
        if !next_path.is_dir() {
            return Ok(false);
        }
        current_dir = next_path;
    }
    Ok(false)
}

fn exact_child_path(
    dir: &Path,
    file_name: &std::ffi::OsStr,
) -> Result<Option<PathBuf>, QianjiClientError> {
    let entries = fs::read_dir(dir).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read Flowhub module directory `{}`: {error}",
            dir.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read Flowhub module directory entry under `{}`: {error}",
                dir.display()
            ))
        })?;
        if entry.file_name() == file_name {
            return Ok(Some(entry.path()));
        }
    }
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

fn collect_manifest_paths(
    dir: &Path,
    depth: usize,
    manifest_paths: &mut Vec<PathBuf>,
) -> Result<(), QianjiClientError> {
    if depth > 6 || !dir.is_dir() {
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
            if path
                .file_name()
                .is_some_and(|file_name| file_name == ".git")
            {
                continue;
            }
            collect_manifest_paths(&path, depth + 1, manifest_paths)?;
        } else if path
            .file_name()
            .is_some_and(|file_name| file_name == "qianji.toml")
        {
            manifest_paths.push(path);
        }
    }
    manifest_paths.sort();
    Ok(())
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
        fix: false,
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
