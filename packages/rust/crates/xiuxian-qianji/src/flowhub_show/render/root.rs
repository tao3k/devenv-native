use crate::flowhub::show::api::FlowhubModuleSummary;
use serde_json::json;

use super::{
    FLOWHUB_ROOT_MODULE_TEMPLATE_NAME, module_kind_label, render_embedded_flowhub_block,
    render_scenario_case_section_lines,
};

pub(crate) fn render_flowhub_root_module_section_lines(
    module: &FlowhubModuleSummary,
) -> Vec<String> {
    let mut tail_blocks = Vec::new();
    if !module.child_modules.is_empty() {
        tail_blocks.push(format!(
            "Children:\n{}",
            module
                .child_modules
                .iter()
                .map(|child| format!("- {child}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if !module.scenario_cases.is_empty() {
        tail_blocks.push(format!(
            "Scenario cases:\n{}",
            render_scenario_case_section_lines(&module.module_ref, &module.scenario_cases)
                .join("\n")
        ));
    }

    render_embedded_flowhub_block(
        FLOWHUB_ROOT_MODULE_TEMPLATE_NAME,
        json!({
            "path": module.module_dir.display().to_string(),
            "kind": module_kind_label(module.kind),
            "exports_entry": module.exports_entry,
            "exports_ready": module.exports_ready,
            "tail_block": if tail_blocks.is_empty() {
                String::new()
            } else {
                format!("\n{}", tail_blocks.join("\n"))
            },
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub root module section through qianhuan; falling back to inline format: {error}"
        );
        render_flowhub_root_module_section_lines_inline(module)
    })
}

fn render_flowhub_root_module_section_lines_inline(module: &FlowhubModuleSummary) -> Vec<String> {
    let mut lines = vec![
        format!("Path: {}", module.module_dir.display()),
        format!("Kind: {}", module_kind_label(module.kind)),
        format!(
            "Exports: {} -> {}",
            module.exports_entry, module.exports_ready
        ),
    ];
    if !module.child_modules.is_empty() {
        lines.push("Children:".to_string());
        lines.extend(
            module
                .child_modules
                .iter()
                .map(|child| format!("- {child}")),
        );
    }
    if !module.scenario_cases.is_empty() {
        lines.push("Scenario cases:".to_string());
        lines.extend(render_scenario_case_section_lines(
            &module.module_ref,
            &module.scenario_cases,
        ));
    }
    lines
}
