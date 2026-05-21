use std::io;

use xiuxian_qianji_control::{RunView, StepView};

use super::serde_status;

pub(crate) fn render_run_view_text(view: &RunView) -> String {
    let step_activity_count = view
        .steps
        .values()
        .map(|step| step.activities.len())
        .sum::<usize>();
    let step_timer_count = view
        .steps
        .values()
        .map(|step| step.timers.len())
        .sum::<usize>();
    let mut output = format!(
        concat!(
            "# Qianji Control View\n\n",
            "- Run: `{}`\n",
            "- Status: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Steps: `{}`\n",
            "- Run activities: `{}`\n",
            "- Step activities: `{}`\n",
            "- Run timers: `{}`\n",
            "- Step timers: `{}`\n",
            "- Signals: `{}`\n",
            "- Total cost usd micros: `{}`\n"
        ),
        view.run_id.as_str(),
        serde_status(&view.status),
        view.updated_at_ms,
        view.steps.len(),
        view.activities.len(),
        step_activity_count,
        view.timers.len(),
        step_timer_count,
        view.signals.len(),
        view.total_cost_usd_micros()
    );

    if !view.steps.is_empty() {
        output.push_str("\n## Steps\n\n");
        for step in view.steps.values() {
            render_step_summary(&mut output, step);
        }
    }

    output
}

pub(crate) fn render_run_view_json(view: &RunView) -> io::Result<String> {
    serde_json::to_string_pretty(view).map_err(io::Error::other)
}

pub(crate) fn render_step_view_text(step: &StepView) -> String {
    let title = step.title.as_deref().unwrap_or("<untitled>");
    format!(
        concat!(
            "# Qianji Control Step\n\n",
            "- Step: `{}`\n",
            "- Status: `{}`\n",
            "- Title: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Required evidence: `{}`\n",
            "- Covered evidence: `{}`\n",
            "- Evidence refs: `{}`\n",
            "- Artifacts: `{}`\n",
            "- Activities: `{}`\n",
            "- Agent proposals: `{}`\n",
            "- Agent decisions: `{}`\n",
            "- Gates: `{}`\n",
            "- Timers: `{}`\n",
            "- Signals: `{}`\n",
            "- Version pins: `{}`\n",
            "- Recovery attempts: `{}`\n",
            "- Total cost usd micros: `{}`\n"
        ),
        step.step_id.as_str(),
        serde_status(&step.status),
        title,
        step.updated_at_ms,
        step.required_evidence.len(),
        step.covered_required_evidence().len(),
        step.evidence.len(),
        step.artifacts.len(),
        step.activities.len(),
        step.agent_proposals.len(),
        step.agent_decisions.len(),
        step.gate_results.len(),
        step.timers.len(),
        step.signals.len(),
        step.version_pins.len(),
        step.recovery_attempts.len(),
        step.total_cost_usd_micros()
    )
}

pub(crate) fn render_step_view_json(step: &StepView) -> io::Result<String> {
    serde_json::to_string_pretty(step).map_err(io::Error::other)
}

fn render_step_summary(output: &mut String, step: &StepView) {
    let title = step.title.as_deref().unwrap_or("<untitled>");
    output.push_str("- `");
    output.push_str(step.step_id.as_str());
    output.push_str("` [");
    output.push_str(&serde_status(&step.status));
    output.push_str("] ");
    output.push_str(title);
    output.push_str(" evidence `");
    output.push_str(&step.covered_required_evidence().len().to_string());
    output.push('/');
    output.push_str(&step.required_evidence.len().to_string());
    output.push_str("` activities `");
    output.push_str(&step.activities.len().to_string());
    output.push_str("` gates `");
    output.push_str(&step.gate_results.len().to_string());
    output.push_str("` updated `");
    output.push_str(&step.updated_at_ms.to_string());
    output.push_str("`\n");
}
