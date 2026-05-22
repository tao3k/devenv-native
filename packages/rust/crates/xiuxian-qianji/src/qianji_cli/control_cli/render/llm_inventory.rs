use std::io;

use xiuxian_qianji_control::LlmActivityInventoryProjection;

use super::{activity_scope_label, push_fmt, serde_status};

pub(crate) fn render_llm_activity_inventory_text(
    projection: &LlmActivityInventoryProjection,
) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control LLM Activities\n\n",
            "- Run: `{}`\n",
            "- Activities: total `{}`, scheduled `{}`, in-flight `{}`, completed `{}`, failed `{}`\n",
            "- Missing request audit: `{}`\n"
        ),
        projection.run_id.as_str(),
        projection.summary.total,
        projection.summary.scheduled,
        projection.summary.in_flight,
        projection.summary.completed,
        projection.summary.failed,
        projection.summary.missing_request_audit
    );

    if !projection.items.is_empty() {
        output.push_str("\n## Activities\n\n");
        for item in &projection.items {
            let model = item.model.as_deref().unwrap_or("<none>");
            let audit = if item.request_audit_metadata.is_null() {
                "missing"
            } else {
                "present"
            };
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` [{}] status `{}` type `{}` queue `{}` model `{}` audit `{}` attempt `{}` updated_at_ms `{}`\n",
                    item.activity_id.as_str(),
                    activity_scope_label(&item.scope),
                    serde_status(&item.status),
                    item.activity_type.as_str(),
                    item.task_queue.as_str(),
                    model,
                    audit,
                    item.attempt,
                    item.updated_at_ms
                ),
            );
        }
    }

    output
}

pub(crate) fn render_llm_activity_inventory_json(
    projection: &LlmActivityInventoryProjection,
) -> io::Result<String> {
    serde_json::to_string_pretty(projection).map_err(io::Error::other)
}
