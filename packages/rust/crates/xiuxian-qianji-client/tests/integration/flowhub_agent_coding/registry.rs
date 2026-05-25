use xiuxian_qianji_client::load_flowhub_scenario_registry;

use super::support::{FlowhubTestProject, copy_agent_coding_pair, rendered_json, run};

#[test]
fn contract_accepts_org_bpmn_source_pair_without_manifest() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");

    let output = run(
        &project.init_args("agent-coding", None, false),
        "Org+BPMN source-pair init",
    );
    assert!(output.passed, "{}", output.rendered);
    assert!(output.rendered.contains("Flowhub contract: passed"));
}

#[test]
fn contract_rejects_module_manifest_without_policy_entry() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    std::fs::write(
        project.flowhub_root.join("plan/qianji.toml"),
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "agent-coding.org",
  "agent-coding.bpmn",
]
"#,
    )
    .unwrap_or_else(|error| panic!("module manifest should write: {error}"));

    let output = run(
        &project.init_args("agent-coding", None, false),
        "missing policy entry lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("must list required policy entry `PLAN_POLICY.org`")
    );
    assert!(output.rendered.contains("is missing required policy entry"));
}

#[test]
fn contract_rejects_wrong_case_module_policy_entry() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    std::fs::write(
        project.flowhub_root.join("plan/qianji.toml"),
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
]
"#,
    )
    .unwrap_or_else(|error| panic!("module manifest should write: {error}"));
    std::fs::write(
        project.flowhub_root.join("plan/plan_policy.org"),
        r"#+TITLE: Lowercase Plan Policy

* Policy Entry
:PROPERTIES:
:FLOWHUB_POLICY_ENTRY: module
:FLOWHUB_POLICY_MODE: PLAN
:END:
",
    )
    .unwrap_or_else(|error| panic!("lowercase policy entry should write: {error}"));

    let output = run(
        &project.init_args("agent-coding", None, false),
        "wrong case policy entry lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(output.rendered.contains("is missing required policy entry"));
    assert!(output.rendered.contains("PLAN_POLICY.org"));
}

#[test]
fn contract_rejects_missing_manifest_required_surface() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    write_plan_manifest(
        &project,
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
  "_execplan_template.org",
]
"#,
    );
    write_minimal_plan_policy(&project);

    let output = run(
        &project.init_args("agent-coding", None, false),
        "missing required surface lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("lists missing required surface `_execplan_template.org`")
    );
}

#[test]
fn contract_rejects_manifest_required_surface_outside_module() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    write_plan_manifest(
        &project,
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
  "../PLAN_POLICY.org",
]
"#,
    );
    write_minimal_plan_policy(&project);

    let output = run(
        &project.init_args("agent-coding", None, false),
        "invalid required surface lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("has invalid required surface `../PLAN_POLICY.org`")
    );
}

#[test]
fn contract_accepts_module_policy_entry() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    write_plan_manifest(
        &project,
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
]
"#,
    );
    write_minimal_plan_policy(&project);

    let output = run(
        &project.init_args("agent-coding", None, false),
        "module policy entry lint",
    );

    assert!(output.passed, "{}", output.rendered);
}

#[test]
fn contract_rejects_invalid_module_policy_contract_graph_selector() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    std::fs::write(
        project.flowhub_root.join("plan/qianji.toml"),
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
]
"#,
    )
    .unwrap_or_else(|error| panic!("module manifest should write: {error}"));
    std::fs::write(
        project.flowhub_root.join("plan/PLAN_POLICY.org"),
        r"#+TITLE: Plan Policy

* Policy Entry
:PROPERTIES:
:FLOWHUB_POLICY_ENTRY: module
:FLOWHUB_POLICY_MODE: PLAN
:FLOWHUB_CONTRACT_GRAPH: (:org-element :type src-block :name plan_contract_graph)
:END:
",
    )
    .unwrap_or_else(|error| panic!("policy entry should write: {error}"));

    let output = run(
        &project.init_args("agent-coding", None, false),
        "invalid policy graph selector lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("invalid FLOWHUB_CONTRACT_GRAPH selector")
    );
}

#[test]
fn contract_rejects_missing_module_policy_contract_graph_target() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    std::fs::write(
        project.flowhub_root.join("plan/qianji.toml"),
        r#"version = 1

[module]
name = "plan"

[contract]
required = [
  "PLAN_POLICY.org",
  "agent-coding.org",
  "agent-coding.bpmn",
]
"#,
    )
    .unwrap_or_else(|error| panic!("module manifest should write: {error}"));
    std::fs::write(
        project.flowhub_root.join("plan/PLAN_POLICY.org"),
        r#"#+TITLE: Plan Policy

* Policy Entry
:PROPERTIES:
:FLOWHUB_POLICY_ENTRY: module
:FLOWHUB_POLICY_MODE: PLAN
:FLOWHUB_CONTRACT_GRAPH: (:org-element (:type src-block :name "plan_contract_graph" :language "mermaid"))
:END:

#+begin_src mermaid
flowchart LR
  P["PLAN_POLICY.org"] --> T["_execplan_template.org"]
#+end_src
"#,
    )
    .unwrap_or_else(|error| panic!("policy entry should write: {error}"));

    let output = run(
        &project.init_args("agent-coding", None, false),
        "missing policy graph target lint",
    );

    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("FLOWHUB_CONTRACT_GRAPH selector matched no Org element")
    );
}

#[test]
fn scenarios_lists_org_bpmn_source_pairs() {
    let project = FlowhubTestProject::live();

    let output = run(&project.scenarios_args(false), "scenario listing");
    assert!(output.passed, "{}", output.rendered);
    assert!(
        output
            .source_pairs
            .iter()
            .any(|source_pair| source_pair.scenario_id == "agent-coding")
    );
    assert!(
        output
            .source_pairs
            .iter()
            .any(|source_pair| source_pair.scenario_id == "deep_read")
    );
    assert!(
        output
            .source_pairs
            .iter()
            .any(|source_pair| source_pair.scenario_id == "wendao-client-plan-policy")
    );
    assert!(
        output
            .rendered
            .contains("# Qianji Client Flowhub Scenarios")
    );
    assert!(output.rendered.contains("`deep_read`"));
}

#[test]
fn scenarios_rejects_duplicate_scenario_ids() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");
    copy_agent_coding_pair(&project, "other");

    let output = run(&project.scenarios_args(false), "duplicate scenario lint");
    assert!(!output.passed, "{}", output.rendered);
    assert!(
        output
            .rendered
            .contains("duplicate Flowhub scenario id `agent-coding`")
    );
}

#[test]
fn scenarios_json_renders_machine_readable_registry() {
    let project = FlowhubTestProject::live();

    let output = run(&project.scenarios_args(true), "scenario JSON listing");
    assert!(output.passed, "{}", output.rendered);
    let rendered = rendered_json(&output);
    assert_eq!(rendered["action"], "scenarios");
    assert_eq!(rendered["passed"], true);
    assert!(
        rendered["sourcePairs"]
            .as_array()
            .unwrap_or_else(|| panic!("sourcePairs should be an array"))
            .iter()
            .any(|source_pair| source_pair["scenarioId"] == "deep_read"
                && source_pair["orgSha256"]
                    .as_str()
                    .is_some_and(|hash| hash.len() == 64)
                && source_pair["bpmnSha256"]
                    .as_str()
                    .is_some_and(|hash| hash.len() == 64)
                && source_pair["bpmnProcessId"] == "paper_deep_read")
    );
    assert!(
        rendered["sourcePairs"]
            .as_array()
            .unwrap_or_else(|| panic!("sourcePairs should be an array"))
            .iter()
            .any(
                |source_pair| source_pair["scenarioId"] == "wendao-client-plan-policy"
                    && source_pair["bpmnProcessId"] == "wendao_client_plan_policy"
            )
    );
}

#[test]
fn registry_api_returns_machine_readable_source_pairs() {
    let project = FlowhubTestProject::live();

    let registry = load_flowhub_scenario_registry(&project.flowhub_root)
        .unwrap_or_else(|error| panic!("registry API should load source pairs: {error}"));

    assert!(registry.passed, "{:?}", registry.validation.diagnostics);
    assert_eq!(registry.action, "scenarios");
    assert!(
        registry.source_pairs.iter().any(|source_pair| {
            source_pair.scenario_id == "deep_read"
                && source_pair.org_sha256.len() == 64
                && source_pair.bpmn_sha256.len() == 64
                && source_pair.bpmn_process_id == "paper_deep_read"
        }),
        "registry should include deep_read source pair: {registry:?}"
    );
    assert!(
        registry.source_pairs.iter().any(|source_pair| {
            source_pair.scenario_id == "wendao-client-plan-policy"
                && source_pair.bpmn_process_id == "wendao_client_plan_policy"
        }),
        "registry should include Wendao client plan policy source pair: {registry:?}"
    );
}

fn write_plan_manifest(project: &FlowhubTestProject, source: &str) {
    std::fs::write(project.flowhub_root.join("plan/qianji.toml"), source)
        .unwrap_or_else(|error| panic!("module manifest should write: {error}"));
}

fn write_minimal_plan_policy(project: &FlowhubTestProject) {
    std::fs::write(
        project.flowhub_root.join("plan/PLAN_POLICY.org"),
        r#"#+TITLE: Plan Policy

* Policy Entry
:PROPERTIES:
:FLOWHUB_POLICY_ENTRY: module
:FLOWHUB_POLICY_MODE: PLAN
:FLOWHUB_CONTRACT_GRAPH: (:org-element (:type src-block :name "plan_contract_graph" :language "mermaid"))
:END:

#+name: plan_contract_graph
#+begin_src mermaid
flowchart LR
  P["PLAN_POLICY.org"] --> T["_execplan_template.org"]
#+end_src
"#,
    )
    .unwrap_or_else(|error| panic!("uppercase policy entry should write: {error}"));
}
