//! Agent tracking materialization for Flowhub plan scenarios.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::{OrgizeLintOutputFormat, OrgizeLintRequest, lint_org_files};

use super::contract::{
    FlowhubSourcePair, list_flowhub_source_pairs, parse_org_properties, property_value,
    resolve_flowhub_source_pair, validate_flowhub_source_pair_contract,
};
use super::model::{
    FlowhubCheckStatus, FlowhubCliOutput, FlowhubGeneratedFile, FlowhubSourcePairSummary,
    FlowhubValidation,
};
use super::parse::{FlowhubAction, FlowhubCommand};
use super::render::{RenderInput, render_output};
use crate::QianjiClientError;

const EMBEDDED_AGENT_CODING_ORG: &str =
    include_str!("../../../../../../qianji-flowhub/plan/agent-coding.org");
const EMBEDDED_AGENT_CODING_BPMN: &str =
    include_str!("../../../../../../qianji-flowhub/plan/agent-coding.bpmn");

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentPlanSourceMetadata {
    scenario_id: String,
    org_source: String,
    org_sha256: String,
    bpmn_source: String,
    bpmn_sha256: String,
    bpmn_process_id: String,
}

pub(crate) fn run_flowhub_plan(
    command: FlowhubCommand,
) -> Result<FlowhubCliOutput, QianjiClientError> {
    let agent_root = command.cache_home.join("agent");
    let generated_files = match command.action {
        FlowhubAction::Init => materialize_agent_tracking_files(&command, &agent_root)?,
        FlowhubAction::Check => expected_generated_files(&command, &agent_root)
            .into_iter()
            .map(|path| FlowhubGeneratedFile {
                path,
                created: false,
            })
            .collect(),
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
    if let Some(flowhub_root) = command.flowhub_root.as_deref() {
        let mut diagnostics = Vec::new();
        if let Some(source_pair) =
            resolve_flowhub_source_pair(flowhub_root, scenario, &mut diagnostics)?
        {
            return Ok(AgentPlanSourceMetadata {
                scenario_id: source_pair.scenario_id,
                org_source: source_pair.org_source.display().to_string(),
                org_sha256: sha256_file(&source_pair.org_source, "Flowhub Org source")?,
                bpmn_source: source_pair.bpmn_source.display().to_string(),
                bpmn_sha256: sha256_file(&source_pair.bpmn_source, "Flowhub BPMN source")?,
                bpmn_process_id: source_pair.bpmn_process_id,
            });
        }
    }

    if scenario == "agent-coding" {
        return Ok(AgentPlanSourceMetadata {
            scenario_id: "agent-coding".to_string(),
            org_source: "qianji-flowhub/plan/agent-coding.org".to_string(),
            org_sha256: sha256_text(EMBEDDED_AGENT_CODING_ORG),
            bpmn_source: "qianji-flowhub/plan/agent-coding.bpmn".to_string(),
            bpmn_sha256: sha256_text(EMBEDDED_AGENT_CODING_BPMN),
            bpmn_process_id: "agent_coding".to_string(),
        });
    }

    Ok(AgentPlanSourceMetadata {
        scenario_id: scenario.to_string(),
        org_source: "unresolved".to_string(),
        org_sha256: "unresolved".to_string(),
        bpmn_source: "unresolved".to_string(),
        bpmn_sha256: "unresolved".to_string(),
        bpmn_process_id: "unresolved".to_string(),
    })
}

fn sha256_file(path: &Path, label: &str) -> Result<String, QianjiClientError> {
    let source = fs::read_to_string(path).map_err(|error| {
        QianjiClientError::message(format!(
            "Failed to read {label} `{}` for hashing: {error}",
            path.display()
        ))
    })?;
    Ok(sha256_text(&source))
}

fn sha256_text(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
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
        FlowhubAction::Init | FlowhubAction::Check => validate_flowhub_contract(
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
    let generated_metadata_matched = if matches!(command.action, FlowhubAction::Scenarios) {
        true
    } else if flowhub_contract_passed && generated_files_present {
        validate_generated_source_metadata(
            command,
            &validation_scenario,
            generated_files,
            &mut diagnostics,
        )?
    } else {
        false
    };

    Ok(FlowhubValidation {
        flowhub_contract: FlowhubCheckStatus::from_bool(flowhub_contract_passed),
        generated_files: FlowhubCheckStatus::from_bool(generated_files_present),
        generated_metadata: FlowhubCheckStatus::from_bool(generated_metadata_matched),
        org_lint: FlowhubCheckStatus::from_bool(org_lint_passed),
        diagnostics,
    })
}

fn validation_scenario_for_generated_files(
    command: &FlowhubCommand,
    generated_files: &[FlowhubGeneratedFile],
    generated_files_present: bool,
) -> String {
    if command.action != FlowhubAction::Check || !generated_files_present {
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
            .push("flowhub scenarios requires --flowhub-root or QIANJI_FLOWHUB_ROOT".to_string());
        return Ok(false);
    };
    let pairs = list_flowhub_source_pairs(flowhub_root, diagnostics)?;
    let mut passed = !pairs.is_empty();
    for pair in &pairs {
        passed &=
            validate_flowhub_source_pair_contract(flowhub_root, &pair.scenario_id, diagnostics)?;
    }
    source_pairs.extend(pairs.into_iter().map(source_pair_summary));
    if source_pairs.is_empty() {
        diagnostics.push(format!(
            "Flowhub root `{}` has no Org+BPMN source pairs",
            flowhub_root.display()
        ));
    }
    Ok(passed)
}

fn source_pair_summary(source_pair: FlowhubSourcePair) -> FlowhubSourcePairSummary {
    FlowhubSourcePairSummary {
        scenario_id: source_pair.scenario_id,
        org_source: source_pair.org_source,
        bpmn_source: source_pair.bpmn_source,
        bpmn_process_id: source_pair.bpmn_process_id,
    }
}

fn validate_flowhub_contract(
    flowhub_root: Option<&Path>,
    scenario: &str,
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    if flowhub_root.is_none() && scenario != "agent-coding" {
        diagnostics.push(format!(
            "scenario `{scenario}` requires --flowhub-root because only `agent-coding` is embedded"
        ));
        return Ok(false);
    }
    if !embedded_agent_coding_contract_is_valid() {
        diagnostics.push("embedded agent-coding contract is incomplete".to_string());
        return Ok(false);
    }

    let Some(flowhub_root) = flowhub_root else {
        return Ok(true);
    };
    validate_flowhub_source_pair_contract(flowhub_root, scenario, diagnostics)
}

fn embedded_agent_coding_contract_is_valid() -> bool {
    EMBEDDED_AGENT_CODING_ORG.contains(":BPMN_SOURCE: agent-coding.bpmn")
        && EMBEDDED_AGENT_CODING_ORG.contains("#+begin_src mermaid")
        && EMBEDDED_AGENT_CODING_BPMN.contains("agent_coding")
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
    let paths = generated_files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let request = OrgizeLintRequest {
        paths,
        output_format: OrgizeLintOutputFormat::Compact,
        priority_highest: None,
        priority_lowest: None,
        priority_default: None,
    };
    let report = lint_org_files(&request).map_err(|error| {
        QianjiClientError::message(format!("Failed to lint generated agent Org files: {error}"))
    })?;
    if report.is_clean() {
        return Ok(true);
    }
    diagnostics.push(report.render(OrgizeLintOutputFormat::Compact));
    Ok(false)
}

fn validate_generated_source_metadata(
    command: &FlowhubCommand,
    validation_scenario: &str,
    generated_files: &[FlowhubGeneratedFile],
    diagnostics: &mut Vec<String>,
) -> Result<bool, QianjiClientError> {
    let expected = source_metadata_for_scenario(command, validation_scenario)?;
    let mut passed = true;
    for file in generated_files {
        let source = fs::read_to_string(&file.path).map_err(|error| {
            QianjiClientError::message(format!(
                "Failed to read generated agent tracking file `{}`: {error}",
                file.path.display()
            ))
        })?;
        let properties = parse_org_properties(&source);
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_SLUG",
            &command.slug,
            diagnostics,
        );
        passed &=
            validate_generated_slug_field(&file.path, &properties, &command.slug, diagnostics);
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_SCENARIO_ID",
            &expected.scenario_id,
            diagnostics,
        );
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_ORG_SOURCE",
            &expected.org_source,
            diagnostics,
        );
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_ORG_SHA256",
            &expected.org_sha256,
            diagnostics,
        );
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_BPMN_SOURCE",
            &expected.bpmn_source,
            diagnostics,
        );
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "FLOWHUB_BPMN_SHA256",
            &expected.bpmn_sha256,
            diagnostics,
        );
        passed &= validate_generated_property(
            &file.path,
            &properties,
            "BPMN_PROCESS_ID",
            &expected.bpmn_process_id,
            diagnostics,
        );
    }
    Ok(passed)
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
) -> bool {
    let key = if path
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "sdd")
    {
        "SDD_SLUG"
    } else {
        "SLICE"
    };
    validate_generated_property(path, properties, key, expected, diagnostics)
}

fn validate_generated_property(
    path: &Path,
    properties: &[(String, String)],
    key: &str,
    expected: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let actual = property_value(properties, key);
    if actual == Some(expected) {
        return true;
    }

    diagnostics.push(format!(
        "generated agent tracking file `{}` has {} `{}` but expected `{}`",
        path.display(),
        key,
        actual.unwrap_or("<missing>"),
        expected
    ));
    false
}

fn expected_generated_files(
    command: &FlowhubCommand,
    agent_root: &Path,
) -> Vec<std::path::PathBuf> {
    let slug = file_slug(&command.slug);
    vec![
        agent_root.join("sdd").join(format!("{slug}.org")),
        agent_root.join("org").join(format!("{slug}.org")),
        agent_root.join("execplans").join(format!("{slug}.org")),
    ]
}

fn render_sdd(
    source: &AgentPlanSourceMetadata,
    slug: &str,
    sdd_path: &Path,
    org_path: &Path,
    execplan_path: &Path,
) -> String {
    let input = SddTemplateInput {
        title: display_title(slug),
        slug,
        source,
        root_id: stable_uuid(slug, "system"),
        capability_id: stable_uuid(slug, "capability"),
        view_id: stable_uuid(slug, "view"),
        decision_id: stable_uuid(slug, "decision"),
        audit_id: stable_uuid(slug, "audit"),
        sdd_path,
        org_path,
        execplan_path,
    };
    let mut rendered = String::new();
    rendered.push_str(&render_sdd_system_section(&input));
    rendered.push_str(&render_sdd_runtime_section(&input));
    rendered.push_str(&render_sdd_decision_section(&input));
    rendered.push_str(&render_sdd_audit_section(&input));
    rendered
}

struct SddTemplateInput<'a> {
    title: String,
    slug: &'a str,
    source: &'a AgentPlanSourceMetadata,
    root_id: String,
    capability_id: String,
    view_id: String,
    decision_id: String,
    audit_id: String,
    sdd_path: &'a Path,
    org_path: &'a Path,
    execplan_path: &'a Path,
}

fn render_sdd_system_section(input: &SddTemplateInput<'_>) -> String {
    format!(
        r"#+TITLE: {title} SDD
#+AUTHOR: CyberXiuXian Artisan workshop
#+FILETAGS: :agent:sdd:qianji_client:

* {title} :sdd:system:
:PROPERTIES:
:ID: {root_id}
:SDD_KIND: system
:SDD_STATUS: draft
:SDD_CONCERN: Agent coding plan materialization and validation for this downstream project.
:SDD_SLUG: {slug}
:FLOWHUB_SLUG: {slug}
:FLOWHUB_SCENARIO_ID: {scenario_id}
:FLOWHUB_ORG_SOURCE: {org_source}
:FLOWHUB_ORG_SHA256: {org_sha256}
:FLOWHUB_BPMN_SOURCE: {bpmn_source}
:FLOWHUB_BPMN_SHA256: {bpmn_sha256}
:BPMN_PROCESS_ID: {bpmn_process_id}
:END:

** Agent Plan Tracking :sdd:capability:
:PROPERTIES:
:ID: {capability_id}
:SDD_KIND: capability
:SDD_PARENT: [[id:{root_id}][{title}]]
:SDD_CAPABILITY: agent-plan-tracking
:SDD_STATUS: draft
:SDD_SLUG: {slug}-tracking
:END:

*** Requirement: Recoverable agent plan
The project SHALL keep active agent implementation work recoverable from Org tracking files.

**** Scenario: Agent resumes work
- WHEN an Agent resumes this project
- THEN the active Org task and linked SDD SHALL expose the current status, scope, and validation evidence.

*** Requirement: Explicit validation surface
The project SHALL record validation commands and evidence before the slice is treated as complete.

**** Scenario: Agent completes implementation
- WHEN implementation is complete
- THEN validation evidence SHALL be recorded in the Org task and linked plan surface.

",
        title = input.title.as_str(),
        root_id = input.root_id.as_str(),
        capability_id = input.capability_id.as_str(),
        scenario_id = input.source.scenario_id.as_str(),
        org_source = input.source.org_source.as_str(),
        org_sha256 = input.source.org_sha256.as_str(),
        bpmn_source = input.source.bpmn_source.as_str(),
        bpmn_sha256 = input.source.bpmn_sha256.as_str(),
        bpmn_process_id = input.source.bpmn_process_id.as_str(),
        slug = input.slug,
    )
}

fn render_sdd_runtime_section(input: &SddTemplateInput<'_>) -> String {
    format!(
        r#"** Runtime View :sdd:view:
:PROPERTIES:
:ID: {view_id}
:SDD_KIND: view
:SDD_PARENT: [[id:{capability_id}][Agent Plan Tracking]]
:SDD_VIEWPOINT: runtime
:SDD_CONCERN: Generated agent planning files and recovery queries.
:SDD_QUALITY: determinism, auditability, recovery
:SDD_STATUS: draft
:SDD_SLUG: {slug}-runtime-view
:END:

*** View Description
This project uses the generated SDD, Org task, and ExecPlan as the active plan surface for the ={scenario_id}= Flowhub scenario.

- Org source: ={org_source}=
- Org source SHA-256: ={org_sha256}=
- BPMN source: ={bpmn_source}=
- BPMN source SHA-256: ={bpmn_sha256}=
- BPMN process: ={bpmn_process_id}=

#+begin_src mermaid
flowchart LR
  Request["work request"] --> SDD["SDD: {sdd_path}"]
  SDD --> Org["Org task: {org_path}"]
  Org --> Plan["ExecPlan: {execplan_path}"]
  Plan --> Evidence["validation evidence"]
#+end_src

"#,
        view_id = input.view_id.as_str(),
        capability_id = input.capability_id.as_str(),
        slug = input.slug,
        scenario_id = input.source.scenario_id.as_str(),
        org_source = input.source.org_source.as_str(),
        org_sha256 = input.source.org_sha256.as_str(),
        bpmn_source = input.source.bpmn_source.as_str(),
        bpmn_sha256 = input.source.bpmn_sha256.as_str(),
        bpmn_process_id = input.source.bpmn_process_id.as_str(),
        sdd_path = input.sdd_path.display(),
        org_path = input.org_path.display(),
        execplan_path = input.execplan_path.display(),
    )
}

fn render_sdd_decision_section(input: &SddTemplateInput<'_>) -> String {
    format!(
        r"** Design Decision :sdd:decision:
:PROPERTIES:
:ID: {decision_id}
:SDD_KIND: decision
:SDD_PARENT: [[id:{view_id}][Runtime View]]
:SDD_STATUS: draft
:SDD_RATIONALE: The Flowhub scenario materializes concrete tracking files so downstream users can initialize the same planning contract.
:SDD_SLUG: {slug}-tracking-decision
:END:

*** Decision
Use this generated tracking surface as the active agent plan contract for the current implementation slice.

*** Consequences
- Consequence: The project can run =qianji-client flowhub check= to validate the generated surface.
- Risk: Manually edited tracking files may fail Org lint until corrected.

",
        decision_id = input.decision_id.as_str(),
        view_id = input.view_id.as_str(),
        slug = input.slug,
    )
}

fn render_sdd_audit_section(input: &SddTemplateInput<'_>) -> String {
    format!(
        r"** Architecture Audit :sdd:audit:
:PROPERTIES:
:ID: {audit_id}
:SDD_KIND: audit
:SDD_PARENT: [[id:{capability_id}][Agent Plan Tracking]]
:SDD_STATUS: draft
:SDD_CONCERN: Evidence required before this generated planning surface is considered healthy.
:SDD_QUALITY: correctness, maintainability
:SDD_SLUG: {slug}-audit
:END:

*** Audit Questions
- Question: Do the generated SDD, Org task, and ExecPlan files exist?
- Question: Do all generated Org files pass Orgize lint?
- Question: Does the active Org task record validation evidence before completion?

*** Fitness Criteria
- Gate: =qianji-client flowhub check= passes.
- Gate: The active Org task records completed validation evidence.

*** Linked Implementation
The paired Org task and ExecPlan are generated beside this SDD.
",
        audit_id = input.audit_id.as_str(),
        capability_id = input.capability_id.as_str(),
        slug = input.slug,
    )
}

fn render_org_task(
    source: &AgentPlanSourceMetadata,
    slug: &str,
    sdd_path: &Path,
    execplan_path: &Path,
) -> String {
    let title = display_title(slug);
    format!(
        r"#+TITLE: {title} Task
#+FILETAGS: :agent:qianji_client:

* TODO {title} [0/6] [0%] :agent:qianji_client:
:PROPERTIES:
:SDD: {sdd_path}
:EXECPLAN: {execplan_path}
:STABLE_REF: {org_source}
:FLOWHUB_SLUG: {slug}
:FLOWHUB_SCENARIO_ID: {scenario_id}
:FLOWHUB_ORG_SOURCE: {org_source}
:FLOWHUB_ORG_SHA256: {org_sha256}
:FLOWHUB_BPMN_SOURCE: {bpmn_source}
:FLOWHUB_BPMN_SHA256: {bpmn_sha256}
:BPMN_PROCESS_ID: {bpmn_process_id}
:PACKAGE: downstream-project
:SLICE: {slug}
:STATUS: active
:COMMAND_PROXY: rtk
:COOKIE_DATA: direct
:NEXT_ACTION: Run task-local research and implement the bounded slice.
:RESUME_QUERY: wendao-client orgize task-list --text '{title}' $PRJ_CACHE_HOME/agent/org
:ARCHIVE_TARGET: $PRJ_CACHE_HOME/agent/org/archives/2026.org
:EVIDENCE: pending
:END:

- [ ] Scope and recovery anchor confirmed.
- [ ] RTK command proxy requirement confirmed.
- [ ] Task-local research complete.
- [ ] Implementation complete.
- [ ] Validation complete.
- [ ] Evidence and archive state updated.

** Context

This task was generated by =qianji-client flowhub --mode plan --scenario {scenario_id} init=.
It is bound to the selected Org+BPMN source pair:

- Org source: ={org_source}=
- Org source SHA-256: ={org_sha256}=
- BPMN source: ={bpmn_source}=
- BPMN source SHA-256: ={bpmn_sha256}=
- BPMN process: ={bpmn_process_id}=

** Validation

- [ ] Targeted validation commands pass.
- [ ] =qianji-client flowhub check= passes.
- [ ] Evidence is recorded before completion.

** Evidence

Pending implementation.

** Recovery

#+begin_src text
rtk wendao-client orgize task-list --text '{title}' $PRJ_CACHE_HOME/agent/org
#+end_src
",
        sdd_path = sdd_path.display(),
        execplan_path = execplan_path.display(),
        scenario_id = source.scenario_id,
        org_source = source.org_source,
        org_sha256 = source.org_sha256,
        bpmn_source = source.bpmn_source,
        bpmn_sha256 = source.bpmn_sha256,
        bpmn_process_id = source.bpmn_process_id
    )
}

fn render_execplan(
    source: &AgentPlanSourceMetadata,
    slug: &str,
    sdd_path: &Path,
    org_path: &Path,
) -> String {
    let title = display_title(slug);
    format!(
        r"#+TITLE: {title} ExecPlan
#+FILETAGS: :agent:execplan:qianji_client:

* ExecPlan: {title}
:PROPERTIES:
:SDD_REF: {sdd_path}
:ORG_TASK: {org_path}
:SLICE: {slug}
:FLOWHUB_SLUG: {slug}
:FLOWHUB_SCENARIO_ID: {scenario_id}
:FLOWHUB_ORG_SOURCE: {org_source}
:FLOWHUB_ORG_SHA256: {org_sha256}
:FLOWHUB_BPMN_SOURCE: {bpmn_source}
:FLOWHUB_BPMN_SHA256: {bpmn_sha256}
:BPMN_PROCESS_ID: {bpmn_process_id}
:STATUS: active
:END:

** Goal

Deliver the bounded implementation slice described by the paired SDD and Org task.

** Scope

- Keep the implementation bounded to the active slice.
- Preserve user-owned changes.
- Record validation evidence before completion.

** Plan [0/4] [0%]
:PROPERTIES:
:COOKIE_DATA: direct
:END:

- [ ] Confirm physical target paths.
- [ ] Implement the bounded change.
- [ ] Update or add focused tests.
- [ ] Run validation and record evidence.

** Validation

- [ ] Targeted tests pass.
- [ ] =qianji-client flowhub check= passes.

** Notes

This file was generated by the Flowhub ={scenario_id}= plan scenario.
The selected source pair is ={org_source}= and ={bpmn_source}= with BPMN process ={bpmn_process_id}=.
The source hashes are ={org_sha256}= and ={bpmn_sha256}=.
",
        sdd_path = sdd_path.display(),
        org_path = org_path.display(),
        scenario_id = source.scenario_id,
        org_source = source.org_source,
        org_sha256 = source.org_sha256,
        bpmn_source = source.bpmn_source,
        bpmn_sha256 = source.bpmn_sha256,
        bpmn_process_id = source.bpmn_process_id
    )
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

fn display_title(slug: &str) -> String {
    slug.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn stable_uuid(slug: &str, scope: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(slug.as_bytes());
    hasher.update(b":");
    hasher.update(scope.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!(
        "{}-{}-7{}-8{}-{}",
        &digest[0..8],
        &digest[8..12],
        &digest[13..16],
        &digest[17..20],
        &digest[20..32]
    )
}
