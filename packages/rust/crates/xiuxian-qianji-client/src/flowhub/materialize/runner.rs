//! Agent tracking materialization for Flowhub plan scenarios.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::flowhub::contract::{
    FlowhubSourcePair, list_flowhub_source_pairs, parse_org_properties, property_value,
    resolve_flowhub_source_pair, validate_flowhub_module_policy_entries,
    validate_flowhub_source_pair_contract,
};
use crate::flowhub::model::{
    FlowhubCliOutput, FlowhubGeneratedFile, FlowhubGeneratedMetadataFailure, FlowhubLintStatus,
    FlowhubScenarioRegistry, FlowhubScenarioRegistrySourcePair, FlowhubScenarioRegistryValidation,
    FlowhubSourcePairSummary, FlowhubValidation,
};
use crate::flowhub::org_lint::validate_org_syntax;
use crate::flowhub::parse::{FlowhubAction, FlowhubCommand};
use crate::flowhub::render::{RenderInput, render_output};

use super::template::{render_execplan, render_org_task, render_sdd};
use super::types::AgentPlanSourceMetadata;
use crate::QianjiClientError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedFileGroup {
    slug: String,
    files: Vec<FlowhubGeneratedFile>,
}

pub(crate) fn run_flowhub_plan(
    command: FlowhubCommand,
) -> Result<FlowhubCliOutput, QianjiClientError> {
    let agent_root = command.cache_home.join("agent");
    let generated_files = match command.action {
        FlowhubAction::Init => materialize_agent_tracking_files(&command, &agent_root)?,
        FlowhubAction::Lint if command.lint_all => discovered_generated_file_groups(&agent_root)?
            .into_iter()
            .flat_map(|group| group.files)
            .collect(),
        FlowhubAction::Lint => expected_generated_file_group(&command, &agent_root).files,
        FlowhubAction::Scenarios => Vec::new(),
    };
    let mut source_pairs = Vec::new();
    let validation = validate_agent_coding_plan(&command, &generated_files, &mut source_pairs)?;
    Ok(render_output(RenderInput {
        action: command.action,
        project_root: command.project_root,
        cache_agent_root: agent_root,
        flowhub_root: command.flowhub_root,
        generated_files,
        source_pairs,
        validation,
        output_format: command.output_format,
    }))
}

pub(crate) fn load_registry(
    flowhub_root: &Path,
) -> Result<FlowhubScenarioRegistry, QianjiClientError> {
    let mut diagnostics = Vec::new();
    let pairs = list_flowhub_source_pairs(flowhub_root, &mut diagnostics)?;
    let mut passed = !pairs.is_empty();
    passed &= validate_flowhub_module_policy_entries(flowhub_root, &mut diagnostics)?;
    for pair in &pairs {
        passed &= validate_flowhub_source_pair_contract(
            flowhub_root,
            &pair.scenario_id,
            &mut diagnostics,
        )?;
    }
    let source_pairs = source_pair_summaries(pairs)?;
    if source_pairs.is_empty() {
        diagnostics.push(format!(
            "Flowhub root `{}` has no Org+BPMN source pairs",
            flowhub_root.display()
        ));
    }

    Ok(FlowhubScenarioRegistry {
        action: "scenarios".to_string(),
        passed,
        flowhub_root: flowhub_root.display().to_string(),
        source_pairs: source_pairs.into_iter().map(registry_source_pair).collect(),
        validation: FlowhubScenarioRegistryValidation {
            flowhub_contract_passed: passed,
            diagnostics,
        },
    })
}

fn materialize_agent_tracking_files(
    command: &FlowhubCommand,
    agent_root: &Path,
) -> Result<Vec<FlowhubGeneratedFile>, QianjiClientError> {
    create_agent_dirs(agent_root)?;

    let slug = file_slug(&command.slug);
    let source_metadata = materialization_source_metadata(command)?;
    let sdd_path = agent_root.join("sdd").join(format!("{slug}.org"));
    let org_path = agent_root.join("org").join(format!("{slug}.org"));
    let execplan_path = agent_root.join("execplans").join(format!("{slug}.org"));
    let mut files = Vec::new();

    write_if_missing(
        &sdd_path,
        &render_sdd(
            &source_metadata,
            &command.slug,
            &sdd_path,
            &org_path,
            &execplan_path,
        ),
        &mut files,
    )?;
    write_if_missing(
        &org_path,
        &render_org_task(&source_metadata, &command.slug, &sdd_path, &execplan_path),
        &mut files,
    )?;
    write_if_missing(
        &execplan_path,
        &render_execplan(&source_metadata, &command.slug, &sdd_path, &org_path),
        &mut files,
    )?;

    Ok(files)
}

fn materialization_source_metadata(
    command: &FlowhubCommand,
) -> Result<AgentPlanSourceMetadata, QianjiClientError> {
    source_metadata_for_scenario(command, &command.scenario)
}

fn source_metadata_for_scenario(
    command: &FlowhubCommand,
    scenario: &str,
) -> Result<AgentPlanSourceMetadata, QianjiClientError> {
    let Some(flowhub_root) = command.flowhub_root.as_deref() else {
        return Err(missing_flowhub_root_error(scenario));
    };
    let mut diagnostics = Vec::new();
    let Some(source_pair) = resolve_flowhub_source_pair(flowhub_root, scenario, &mut diagnostics)?
    else {
        return Err(QianjiClientError::message(format!(
            "Flowhub root `{}` has no source pair for scenario `{scenario}`{}",
            flowhub_root.display(),
            diagnostic_suffix(&diagnostics)
        )));
    };
    Ok(AgentPlanSourceMetadata {
        scenario_id: source_pair.scenario_id,
        org_source: source_pair.org_source.display().to_string(),
        org_sha256: sha256_file(&source_pair.org_source, "Flowhub Org source")?,
        bpmn_source: source_pair.bpmn_source.display().to_string(),
        bpmn_sha256: sha256_file(&source_pair.bpmn_source, "Flowhub BPMN source")?,
        bpmn_process_id: source_pair.bpmn_process_id,
    })
}

fn sha256_file(path: &Path, label: &str) -> Result<String, QianjiClientError> {
    let source = fs::read_to_string(path).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read {label} `{}` for hashing: {error}",
            path.display()
        ))
    })?;
    Ok(format!("{:x}", Sha256::digest(source.as_bytes())))
}

fn create_agent_dirs(agent_root: &Path) -> Result<(), QianjiClientError> {
    for relative_dir in ["sdd", "org", "execplans"] {
        let dir = agent_root.join(relative_dir);
        fs::create_dir_all(&dir).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to create agent directory `{}`: {error}",
                dir.display()
            ))
        })?;
    }
    Ok(())
}

fn write_if_missing(
    path: &Path,
    content: &str,
    files: &mut Vec<FlowhubGeneratedFile>,
) -> Result<(), QianjiClientError> {
    if path.exists() {
        files.push(FlowhubGeneratedFile {
            path: path.to_path_buf(),
            created: false,
        });
        return Ok(());
    }

    fs::write(path, content).map_err(|error| {
        QianjiClientError::message(format!("Failed to write `{}`: {error}", path.display()))
    })?;
    files.push(FlowhubGeneratedFile {
        path: path.to_path_buf(),
        created: true,
    });
    Ok(())
}

fn validate_agent_coding_plan(
    command: &FlowhubCommand,
    generated_files: &[FlowhubGeneratedFile],
    source_pairs: &mut Vec<FlowhubSourcePairSummary>,
) -> Result<FlowhubValidation, QianjiClientError> {
    if command.action == FlowhubAction::Lint && command.lint_all {
        return validate_all_agent_plans(command, source_pairs);
    }

    let mut diagnostics = Vec::new();
    let generated_files_present = if matches!(command.action, FlowhubAction::Scenarios) {
        true
    } else {
        validate_generated_files(generated_files, &mut diagnostics)
    };
    let validation_scenario =
        validation_scenario_for_generated_files(command, generated_files, generated_files_present);
    let flowhub_contract_passed = match command.action {
        FlowhubAction::Scenarios => list_flowhub_contract(
            command.flowhub_root.as_deref(),
            &mut diagnostics,
            source_pairs,
        )?,
        FlowhubAction::Init | FlowhubAction::Lint => validate_flowhub_contract(
            command.flowhub_root.as_deref(),
            &validation_scenario,
            &mut diagnostics,
        )?,
    };
    let org_lint_passed = if matches!(command.action, FlowhubAction::Scenarios) {
        true
    } else if generated_files_present {
        validate_generated_org_files(generated_files, &mut diagnostics)?
    } else {
        false
    };
    let generated_metadata_report = if matches!(command.action, FlowhubAction::Scenarios) {
        MetadataValidationReport::passed()
    } else if flowhub_contract_passed && generated_files_present {
        validate_generated_source_metadata(
            command,
            &validation_scenario,
            &command.slug,
            generated_files,
            &mut diagnostics,
        )?
    } else {
        MetadataValidationReport::failed()
    };

    Ok(FlowhubValidation {
        flowhub_contract: FlowhubLintStatus::from_bool(flowhub_contract_passed),
        generated_files: FlowhubLintStatus::from_bool(generated_files_present),
        generated_metadata: FlowhubLintStatus::from_bool(generated_metadata_report.passed),
        org_lint: FlowhubLintStatus::from_bool(org_lint_passed),
        diagnostics,
        generated_metadata_failures: generated_metadata_report.failures,
    })
}

fn validate_all_agent_plans(
    command: &FlowhubCommand,
    source_pairs: &mut Vec<FlowhubSourcePairSummary>,
) -> Result<FlowhubValidation, QianjiClientError> {
    let agent_root = command.cache_home.join("agent");
    let groups = discovered_generated_file_groups(&agent_root)?;
    let mut diagnostics = Vec::new();
    if groups.is_empty() {
        diagnostics.push(format!(
            "no generated agent tracking files found under `{}`",
            agent_root.display()
        ));
        return Ok(FlowhubValidation {
            flowhub_contract: FlowhubLintStatus::Failed,
            generated_files: FlowhubLintStatus::Failed,
            generated_metadata: FlowhubLintStatus::Failed,
            org_lint: FlowhubLintStatus::Failed,
            diagnostics,
            generated_metadata_failures: Vec::new(),
        });
    }

    let mut flowhub_contract_passed = true;
    let mut generated_files_present = true;
    let mut generated_metadata_report = MetadataValidationReport::passed();
    let mut org_lint_passed = true;
    if let Some(flowhub_root) = command.flowhub_root.as_deref() {
        flowhub_contract_passed &=
            validate_flowhub_module_policy_entries(flowhub_root, &mut diagnostics)?;
    }

    for group in groups {
        let group_files_present = validate_generated_files(&group.files, &mut diagnostics);
        generated_files_present &= group_files_present;
        let validation_scenario =
            validation_scenario_for_generated_files(command, &group.files, group_files_present);
        flowhub_contract_passed &= validate_flowhub_source_contract(
            command.flowhub_root.as_deref(),
            &validation_scenario,
            &mut diagnostics,
        )?;
        if group_files_present {
            org_lint_passed &= validate_generated_org_files(&group.files, &mut diagnostics)?;
            let group_report = validate_generated_source_metadata(
                command,
                &validation_scenario,
                &group.slug,
                &group.files,
                &mut diagnostics,
            )?;
            generated_metadata_report.merge(group_report);
        } else {
            generated_metadata_report.passed = false;
        }
    }

    if let Some(flowhub_root) = command.flowhub_root.as_deref() {
        let pairs = list_flowhub_source_pairs(flowhub_root, &mut diagnostics)?;
        source_pairs.extend(source_pair_summaries(pairs)?);
    }

    Ok(FlowhubValidation {
        flowhub_contract: FlowhubLintStatus::from_bool(flowhub_contract_passed),
        generated_files: FlowhubLintStatus::from_bool(generated_files_present),
        generated_metadata: FlowhubLintStatus::from_bool(generated_metadata_report.passed),
        org_lint: FlowhubLintStatus::from_bool(org_lint_passed),
        diagnostics,
        generated_metadata_failures: generated_metadata_report.failures,
    })
}

fn validation_scenario_for_generated_files(
    command: &FlowhubCommand,
    generated_files: &[FlowhubGeneratedFile],
    generated_files_present: bool,
) -> String {
    if command.action != FlowhubAction::Lint || !generated_files_present {
        return command.scenario.clone();
    }

    generated_property_consensus(generated_files, "FLOWHUB_SCENARIO_ID")
        .unwrap_or_else(|| command.scenario.clone())
}

fn list_flowhub_contract(
    flowhub_root: Option<&Path>,
    diagnostics: &mut Vec<String>,
    source_pairs: &mut Vec<FlowhubSourcePairSummary>,
) -> Result<bool, QianjiClientError> {
    let Some(flowhub_root) = flowhub_root else {
        diagnostics
            .push("flowhub scenarios require --flowhub-root or QIANJI_FLOWHUB_ROOT".to_string());
        return Ok(false);
    };
    let pairs = list_flowhub_source_pairs(flowhub_root, diagnostics)?;
    let mut passed = !pairs.is_empty();
    passed &= validate_flowhub_module_policy_entries(flowhub_root, diagnostics)?;
    for pair in &pairs {
        passed &=
            validate_flowhub_source_pair_contract(flowhub_root, &pair.scenario_id, diagnostics)?;
    }
    source_pairs.extend(source_pair_summaries(pairs)?);
    if source_pairs.is_empty() {
        diagnostics.push(format!(
            "Flowhub root `{}` has no Org+BPMN source pairs",
            flowhub_root.display()
        ));
    }
    Ok(passed)
}

fn source_pair_summaries(
    source_pairs: Vec<FlowhubSourcePair>,
) -> Result<Vec<FlowhubSourcePairSummary>, QianjiClientError> {
    source_pairs
        .into_iter()
        .map(source_pair_summary)
        .collect::<Result<Vec<_>, _>>()
}

fn source_pair_summary(
    source_pair: FlowhubSourcePair,
) -> Result<FlowhubSourcePairSummary, QianjiClientError> {
    let org_sha256 = sha256_file(&source_pair.org_source, "Flowhub Org source")?;
    let bpmn_sha256 = sha256_file(&source_pair.bpmn_source, "Flowhub BPMN source")?;
    Ok(FlowhubSourcePairSummary {
        scenario_id: source_pair.scenario_id,
        org_source: source_pair.org_source,
        org_sha256,
        bpmn_source: source_pair.bpmn_source,
        bpmn_sha256,
        bpmn_process_id: source_pair.bpmn_process_id,
    })
}

fn registry_source_pair(
    source_pair: FlowhubSourcePairSummary,
) -> FlowhubScenarioRegistrySourcePair {
    FlowhubScenarioRegistrySourcePair {
        scenario_id: source_pair.scenario_id,
        org_source: source_pair.org_source.display().to_string(),
        org_sha256: source_pair.org_sha256,
        bpmn_source: source_pair.bpmn_source.display().to_string(),
        bpmn_sha256: source_pair.bpmn_sha256,
        bpmn_process_id: source_pair.bpmn_process_id,
    }
}

fn validate_flowhub_contract(
    flowhub_root: Option<&Path>,
    scenario: &str,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let Some(flowhub_root) = flowhub_root else {
        diagnostics.push(missing_flowhub_root_message(scenario));
        return Ok(false);
    };
    let mut passed = validate_flowhub_module_policy_entries(flowhub_root, diagnostics)?;
    passed &= validate_flowhub_source_pair_contract(flowhub_root, scenario, diagnostics)?;
    Ok(passed)
}

fn validate_flowhub_source_contract(
    flowhub_root: Option<&Path>,
    scenario: &str,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let Some(flowhub_root) = flowhub_root else {
        diagnostics.push(missing_flowhub_root_message(scenario));
        return Ok(false);
    };
    validate_flowhub_source_pair_contract(flowhub_root, scenario, diagnostics)
}

fn missing_flowhub_root_error(scenario: &str) -> QianjiClientError {
    QianjiClientError::message(missing_flowhub_root_message(scenario))
}

fn missing_flowhub_root_message(scenario: &str) -> String {
    format!(
        "scenario `{scenario}` requires --flowhub-root or QIANJI_FLOWHUB_ROOT; Flowhub scenarios are external repositories and are not embedded in qianji-client"
    )
}

fn diagnostic_suffix(diagnostics: &[String]) -> String {
    if diagnostics.is_empty() {
        String::new()
    } else {
        format!(": {}", diagnostics.join("; "))
    }
}

fn validate_generated_files(
    generated_files: &[FlowhubGeneratedFile],
    diagnostics: &mut Vec<String>,
) -> bool {
    let mut passed = true;
    for file in generated_files {
        if !file.path.is_file() {
            diagnostics.push(format!(
                "missing generated agent tracking file `{}`",
                file.path.display()
            ));
            passed = false;
        }
    }
    passed
}

fn validate_generated_org_files(
    generated_files: &[FlowhubGeneratedFile],
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let mut passed = true;
    for file in generated_files {
        let source = fs::read_to_string(&file.path).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read generated agent Org file `{}`: {error}",
                file.path.display()
            ))
        })?;
        passed &= validate_org_syntax(&file.path, &source, diagnostics);
    }
    Ok(passed)
}

fn validate_generated_source_metadata(
    command: &FlowhubCommand,
    validation_scenario: &str,
    expected_slug: &str,
    generated_files: &[FlowhubGeneratedFile],
    diagnostics: &mut Vec<String>,
) -> Result<MetadataValidationReport, QianjiClientError> {
    let expected = source_metadata_for_scenario(command, validation_scenario)?;
    let mut report = MetadataValidationReport::passed();
    for file in generated_files {
        let source = fs::read_to_string(&file.path).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read generated agent tracking file `{}`: {error}",
                file.path.display()
            ))
        })?;
        let properties = parse_org_properties(&source);
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_SLUG",
            expected_slug,
            diagnostics,
            &mut report,
        );
        validate_generated_slug_field(
            &file.path,
            &properties,
            expected_slug,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_SCENARIO_ID",
            &expected.scenario_id,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_ORG_SOURCE",
            &expected.org_source,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_ORG_SHA256",
            &expected.org_sha256,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_BPMN_SOURCE",
            &expected.bpmn_source,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_BPMN_SHA256",
            &expected.bpmn_sha256,
            diagnostics,
            &mut report,
        );
        validate_generated_property(
            &file.path,
            &properties,
            "BPMN_PROCESS_ID",
            &expected.bpmn_process_id,
            diagnostics,
            &mut report,
        );
    }
    Ok(report)
}

struct MetadataValidationReport {
    passed: bool,
    failures: Vec<FlowhubGeneratedMetadataFailure>,
}

impl MetadataValidationReport {
    fn passed() -> Self {
        Self {
            passed: true,
            failures: Vec::new(),
        }
    }

    fn failed() -> Self {
        Self {
            passed: false,
            failures: Vec::new(),
        }
    }

    fn push_failure(&mut self, failure: FlowhubGeneratedMetadataFailure) {
        self.passed = false;
        self.failures.push(failure);
    }

    fn merge(&mut self, other: Self) {
        self.passed &= other.passed;
        self.failures.extend(other.failures);
    }
}

fn generated_property_consensus(
    generated_files: &[FlowhubGeneratedFile],
    key: &str,
) -> Option<String> {
    let mut consensus = None;
    for file in generated_files {
        let source = fs::read_to_string(&file.path).ok()?;
        let properties = parse_org_properties(&source);
        let value = property_value(&properties, key)?.to_string();
        match &consensus {
            Some(existing) if existing != &value => return None,
            Some(_) => {}
            None => consensus = Some(value),
        }
    }
    consensus
}

fn validate_generated_slug_field(
    path: &Path,
    properties: &[(String, String)],
    expected: &str,
    diagnostics: &mut Vec<String>,
    report: &mut MetadataValidationReport,
) {
    let key = if path
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "sdd")
    {
        "SDD_SLUG"
    } else {
        "SLICE"
    };
    validate_generated_property(path, properties, key, expected, diagnostics, report);
}

fn validate_generated_property(
    path: &Path,
    properties: &[(String, String)],
    key: &str,
    expected: &str,
    diagnostics: &mut Vec<String>,
    report: &mut MetadataValidationReport,
) {
    let actual = property_value(properties, key);
    if actual == Some(expected) {
        return;
    }

    diagnostics.push(format!(
        "generated agent tracking file `{}` has {} `{}` but expected `{}`",
        path.display(),
        key,
        actual.unwrap_or("<missing>"),
        expected
    ));
    report.push_failure(FlowhubGeneratedMetadataFailure {
        path: path.to_path_buf(),
        key: key.to_string(),
        actual: actual.map(ToString::to_string),
        expected: expected.to_string(),
    });
}

fn expected_generated_file_group(
    command: &FlowhubCommand,
    agent_root: &Path,
) -> GeneratedFileGroup {
    let slug = file_slug(&command.slug);
    GeneratedFileGroup {
        slug: command.slug.clone(),
        files: expected_generated_files_for_stem(agent_root, &slug)
            .into_iter()
            .map(|path| FlowhubGeneratedFile {
                path,
                created: false,
            })
            .collect(),
    }
}

fn discovered_generated_file_groups(
    agent_root: &Path,
) -> Result<Vec<GeneratedFileGroup>, QianjiClientError> {
    let mut stems = BTreeSet::new();
    for relative_dir in ["sdd", "org", "execplans"] {
        let dir = agent_root.join(relative_dir);
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read generated agent directory `{}`: {error}",
                dir.display()
            ))
        })? {
            let entry = entry.map_err(|error| {
                QianjiClientError::message(format!(
                    "Failed to read generated agent directory entry `{}`: {error}",
                    dir.display()
                ))
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "org")
                && generated_file_has_flowhub_slug(&path)?
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                stems.insert(stem.to_string());
            }
        }
    }

    Ok(stems
        .into_iter()
        .map(|stem| {
            let files = expected_generated_files_for_stem(agent_root, &stem)
                .into_iter()
                .map(|path| FlowhubGeneratedFile {
                    path,
                    created: false,
                })
                .collect::<Vec<_>>();
            let slug = generated_property_consensus(&files, "FLOWHUB_SLUG").unwrap_or(stem);
            GeneratedFileGroup { slug, files }
        })
        .collect())
}

fn generated_file_has_flowhub_slug(path: &Path) -> Result<bool, QianjiClientError> {
    let source = fs::read_to_string(path).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read generated agent tracking file `{}`: {error}",
            path.display()
        ))
    })?;
    let properties = parse_org_properties(&source);
    Ok(property_value(&properties, "FLOWHUB_SLUG").is_some())
}

fn expected_generated_files_for_stem(agent_root: &Path, slug: &str) -> Vec<PathBuf> {
    vec![
        agent_root.join("sdd").join(format!("{slug}.org")),
        agent_root.join("org").join(format!("{slug}.org")),
        agent_root.join("execplans").join(format!("{slug}.org")),
    ]
}
fn file_slug(slug: &str) -> String {
    slug.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
