use crate::error::QianjiError;
use crate::flowhub::show::FlowhubScenarioCaseSummary;
use serde_json::json;

use super::{FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME, render_embedded_flowhub_block};

pub(crate) fn render_scenario_case_section_lines(
    module_ref: &str,
    summaries: &[FlowhubScenarioCaseSummary],
) -> Vec<String> {
    let mut lines = Vec::new();
    extend_scenario_case_summary_lines(&mut lines, module_ref, summaries);
    lines
}

pub(crate) fn render_scenario_case_summary_block(
    module_ref: &str,
    summary: &FlowhubScenarioCaseSummary,
) -> Result<String, QianjiError> {
    render_embedded_flowhub_block(
        FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME,
        json!({
            "merimind_graph_name": summary.merimind_graph_name,
            "path": format!("./{module_ref}/{}", summary.file_name),
        }),
    )
    .map(|lines| lines.join("\n"))
    .map_err(|error| {
        QianjiError::Execution(format!(
            "failed to render Flowhub scenario case `{}`: {error}",
            summary.file_name
        ))
    })
}

fn extend_scenario_case_summary_lines(
    lines: &mut Vec<String>,
    module_ref: &str,
    summaries: &[FlowhubScenarioCaseSummary],
) {
    for (index, summary) in summaries.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        match render_scenario_case_summary_block(module_ref, summary) {
            Ok(rendered) => lines.extend(rendered.lines().map(ToOwned::to_owned)),
            Err(error) => {
                log::warn!(
                    "failed to render Flowhub scenario-case markdown through qianhuan; falling back to inline format: {error}"
                );
                lines.push(format!("Graph name: {}", summary.merimind_graph_name));
                lines.push(format!("Path: ./{module_ref}/{}", summary.file_name));
            }
        }
    }
}
