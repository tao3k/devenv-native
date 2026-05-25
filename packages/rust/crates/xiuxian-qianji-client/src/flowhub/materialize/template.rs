//! Org tracking templates for Flowhub plan materialization.

use std::path::Path;

use chrono::Local;
use sha2::{Digest, Sha256};

use super::types::AgentPlanSourceMetadata;

pub(super) fn render_sdd(
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
    let generated_date = generated_org_date();
    format!(
        r"#+TITLE: {title} SDD
#+AUTHOR: CyberXiuXian Artisan workshop
#+FILETAGS: :agent:sdd:xiuxian_qianji_client:
#+DATE: {generated_date}

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
        generated_date = generated_date.as_str(),
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
- Consequence: The project can run =qianji-client flowhub lint= to validate the generated surface.
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
- Gate: =qianji-client flowhub lint= passes.
- Gate: The active Org task records completed validation evidence.

*** Linked Implementation
The paired Org task and ExecPlan are generated beside this SDD.
",
        audit_id = input.audit_id.as_str(),
        capability_id = input.capability_id.as_str(),
        slug = input.slug,
    )
}

pub(super) fn render_org_task(
    source: &AgentPlanSourceMetadata,
    slug: &str,
    sdd_path: &Path,
    execplan_path: &Path,
) -> String {
    let title = display_title(slug);
    let generated_date = generated_org_date();
    format!(
        r"#+TITLE: {title} Task
#+AUTHOR: CyberXiuXian Artisan workshop
#+FILETAGS: :agent:xiuxian_qianji_client:
#+DATE: {generated_date}

* TODO {title} [0/6] [0%] :agent:xiuxian_qianji_client:
:PROPERTIES:
:SDD: {sdd_path}
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
The paired ExecPlan is ={execplan_path}=.
It is bound to the selected Org+BPMN source pair:

- Org source: ={org_source}=
- Org source SHA-256: ={org_sha256}=
- BPMN source: ={bpmn_source}=
- BPMN source SHA-256: ={bpmn_sha256}=
- BPMN process: ={bpmn_process_id}=

** Validation

- [ ] Targeted validation commands pass.
- [ ] =qianji-client flowhub lint= passes.
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
        bpmn_process_id = source.bpmn_process_id,
        generated_date = generated_date.as_str(),
    )
}

pub(super) fn render_execplan(
    source: &AgentPlanSourceMetadata,
    slug: &str,
    sdd_path: &Path,
    org_path: &Path,
) -> String {
    let title = display_title(slug);
    let generated_date = generated_org_date();
    format!(
        r"#+TITLE: {title} ExecPlan
#+AUTHOR: CyberXiuXian Artisan workshop
#+FILETAGS: :agent:execplan:xiuxian_qianji_client:
#+DATE: {generated_date}

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
- [ ] =qianji-client flowhub lint= passes.

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
        bpmn_process_id = source.bpmn_process_id,
        generated_date = generated_date.as_str(),
    )
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

fn generated_org_date() -> String {
    Local::now().format("%Y-%m-%d %a %H:%M:%S").to_string()
}
