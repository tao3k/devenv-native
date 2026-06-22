//! Unit tests for Flowhub markdown show rendering.

use super::{
    FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubScenarioCaseSummary,
    render_flowhub_module_contract_section_lines, render_flowhub_root_module_section_lines,
    render_scenario_case_section_lines, render_scenario_case_summary_block,
};
use std::path::PathBuf;

#[test]
fn scenario_case_summary_block_renders_with_template_catalog()
-> Result<(), crate::error::QianjiError> {
    let summary = FlowhubScenarioCaseSummary {
        file_name: "codex-plan.mmd".to_string(),
        merimind_graph_name: "codex-plan".to_string(),
    };

    let rendered = render_scenario_case_summary_block("plan", &summary)?;

    assert_eq!(
        rendered,
        "Graph name: codex-plan\nPath: ./plan/codex-plan.mmd"
    );
    Ok(())
}

#[test]
fn scenario_case_section_lines_keep_blank_line_between_cases() {
    let lines = render_scenario_case_section_lines(
        "plan",
        &[
            FlowhubScenarioCaseSummary {
                file_name: "codex-plan.mmd".to_string(),
                merimind_graph_name: "codex-plan".to_string(),
            },
            FlowhubScenarioCaseSummary {
                file_name: "review-plan.mmd".to_string(),
                merimind_graph_name: "review-plan".to_string(),
            },
        ],
    );

    assert_eq!(
        lines,
        vec![
            "Graph name: codex-plan".to_string(),
            "Path: ./plan/codex-plan.mmd".to_string(),
            String::new(),
            "Graph name: review-plan".to_string(),
            "Path: ./plan/review-plan.mmd".to_string(),
        ]
    );
}

#[test]
fn flowhub_root_module_section_lines_render_with_template_catalog() {
    let lines = render_flowhub_root_module_section_lines(&FlowhubModuleSummary {
        module_ref: "plan".to_string(),
        module_name: "plan".to_string(),
        module_dir: PathBuf::from("qianji-flowhub/plan"),
        kind: FlowhubModuleKind::Leaf,
        exports_entry: "task.plan-start".to_string(),
        exports_ready: "task.plan-ready".to_string(),
        child_modules: Vec::new(),
        scenario_cases: vec![FlowhubScenarioCaseSummary {
            file_name: "codex-plan.mmd".to_string(),
            merimind_graph_name: "codex-plan".to_string(),
        }],
    });

    assert_eq!(
        lines,
        vec![
            "Path: qianji-flowhub/plan".to_string(),
            "Kind: node".to_string(),
            "Exports: task.plan-start -> task.plan-ready".to_string(),
            "Scenario cases:".to_string(),
            "Graph name: codex-plan".to_string(),
            "Path: ./plan/codex-plan.mmd".to_string(),
        ]
    );
}

#[test]
fn flowhub_module_contract_section_lines_render_with_template_catalog() {
    let lines = render_flowhub_module_contract_section_lines(&FlowhubModuleShow {
        summary: FlowhubModuleSummary {
            module_ref: "plan".to_string(),
            module_name: "plan".to_string(),
            module_dir: PathBuf::from("qianji-flowhub/plan"),
            kind: FlowhubModuleKind::Leaf,
            exports_entry: "task.plan-start".to_string(),
            exports_ready: "task.plan-ready".to_string(),
            child_modules: Vec::new(),
            scenario_cases: Vec::new(),
        },
        registered_child_count: 0,
        required_contract_count: 1,
        scenario_cases: Vec::new(),
    });

    assert_eq!(
        lines,
        vec![
            "Registered children: 0".to_string(),
            "Required contract entries: 1".to_string(),
        ]
    );
}
