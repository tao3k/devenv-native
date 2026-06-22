//! Unit tests for Flowhub scenario markdown show rendering.

use std::path::PathBuf;

use super::{
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview,
    render_scenario_flowchart_section_lines, render_scenario_hidden_aliases_section_lines,
    render_scenario_links_section_lines, render_scenario_surface_section_lines,
};

#[test]
fn scenario_flowchart_section_lines_render_with_template_catalog() {
    let lines = render_scenario_flowchart_section_lines(&FlowhubScenarioShow {
        plan_name: "demo".to_string(),
        scenario_dir: PathBuf::from("/tmp/demo"),
        flowhub_root: PathBuf::from("/tmp/flowhub"),
        flowchart_preview: "flowchart LR\n  blueprint --> plan\n".to_string(),
        surfaces: Vec::new(),
        hidden_aliases: Vec::new(),
        links: Vec::new(),
    });

    assert_eq!(
        lines,
        vec![
            "Status: preview".to_string(),
            "Preview:".to_string(),
            "```mermaid".to_string(),
            "flowchart LR".to_string(),
            "  blueprint --> plan".to_string(),
            "```".to_string(),
        ]
    );
}

#[test]
fn scenario_surface_section_lines_render_with_template_catalog() {
    let lines = render_scenario_surface_section_lines(&FlowhubScenarioSurfacePreview {
        alias: "plan".to_string(),
        module_ref: "plan".to_string(),
        target_path: PathBuf::from("/tmp/scenario/plan"),
        source_manifest_path: PathBuf::from("/tmp/flowhub/plan/qianji.toml"),
    });

    assert_eq!(
        lines,
        vec![
            "Module: plan".to_string(),
            "Target Path: /tmp/scenario/plan".to_string(),
            "Source Manifest: /tmp/flowhub/plan/qianji.toml".to_string(),
        ]
    );
}

#[test]
fn scenario_hidden_aliases_and_links_render_with_template_catalog() {
    let hidden_lines = render_scenario_hidden_aliases_section_lines(&[
        FlowhubScenarioHiddenAlias {
            alias: "constraints".to_string(),
            module_ref: "rust".to_string(),
        },
        FlowhubScenarioHiddenAlias {
            alias: "review".to_string(),
            module_ref: "blueprint".to_string(),
        },
    ]);
    let link_lines = render_scenario_links_section_lines(&[
        "blueprint::ready -> plan::start".to_string(),
        "plan::ready -> coding::done".to_string(),
    ]);

    assert_eq!(
        hidden_lines,
        vec![
            "- constraints -> rust".to_string(),
            "- review -> blueprint".to_string(),
        ]
    );
    assert_eq!(
        link_lines,
        vec![
            "- blueprint::ready -> plan::start".to_string(),
            "- plan::ready -> coding::done".to_string(),
        ]
    );
}
