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

    let output = run(&project.scenarios_args(false), "duplicate scenario check");
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
            .any(|source_pair| source_pair["scenarioId"] == "deep_read")
    );
}
