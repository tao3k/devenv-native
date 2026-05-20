use std::io;

use xiuxian_qianji_control::{ActivityQueueProjection, ActivityView, CostInventoryProjection};

use super::{activity_scope_label, push_fmt, serde_status};

pub(crate) fn render_activity_view_text(activity: &ActivityView) -> String {
    let task_type = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.activity_type.as_str());
    let task_queue = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.task_queue.as_str());
    let idempotency_key = activity
        .task
        .as_ref()
        .map_or("<unknown>", |task| task.idempotency_key.as_str());
    let timeout_ms = activity
        .task
        .as_ref()
        .and_then(|task| task.timeout_ms)
        .map_or_else(|| "<none>".to_string(), |timeout| timeout.to_string());
    let worker = activity
        .worker_id
        .as_ref()
        .map_or("<none>", |worker| worker.as_str());
    let result_hash = activity
        .result
        .as_ref()
        .and_then(|result| result.output_hash.as_deref())
        .unwrap_or("<none>");
    let failure_code = activity
        .failure
        .as_ref()
        .map_or("<none>", |failure| failure.error_code.as_str());
    let failure_retryable = activity
        .failure
        .as_ref()
        .map_or("<none>".to_string(), |failure| {
            failure.retryable.to_string()
        });

    format!(
        concat!(
            "# Qianji Control Activity\n\n",
            "- Activity: `{}`\n",
            "- Status: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Activity type: `{}`\n",
            "- Task queue: `{}`\n",
            "- Idempotency key: `{}`\n",
            "- Timeout ms: `{}`\n",
            "- Attempt: `{}`\n",
            "- Worker: `{}`\n",
            "- Result hash: `{}`\n",
            "- Failure code: `{}`\n",
            "- Failure retryable: `{}`\n"
        ),
        activity.activity_id.as_str(),
        serde_status(&activity.status),
        activity.updated_at_ms,
        task_type,
        task_queue,
        idempotency_key,
        timeout_ms,
        activity.attempt,
        worker,
        result_hash,
        failure_code,
        failure_retryable
    )
}

pub(crate) fn render_activity_view_json(activity: &ActivityView) -> io::Result<String> {
    serde_json::to_string_pretty(activity).map_err(io::Error::other)
}

pub(crate) fn render_activity_queue_projection_text(
    projection: &ActivityQueueProjection,
) -> String {
    let task_queue = projection
        .task_queue
        .as_ref()
        .map_or("<all>", |queue| queue.as_str());
    let mut output = format!(
        concat!(
            "# Qianji Control Activity Queue\n\n",
            "- Run: `{}`\n",
            "- Task queue: `{}`\n",
            "- Queue items: `{}`\n",
            "- Activities: total `{}`, scheduled `{}`, in-flight `{}`, completed `{}`, failed `{}`\n"
        ),
        projection.run_id.as_str(),
        task_queue,
        projection.items.len(),
        projection.summary.total,
        projection.summary.scheduled,
        projection.summary.in_flight,
        projection.summary.completed,
        projection.summary.failed
    );

    if !projection.items.is_empty() {
        output.push_str("\n## Activities\n\n");
        for item in &projection.items {
            let task = item.activity.task.as_ref();
            let activity_type = task.map_or("<unknown>", |task| task.activity_type.as_str());
            let queue = task.map_or("<unknown>", |task| task.task_queue.as_str());
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` [{}] type `{}` queue `{}`\n",
                    item.activity.activity_id.as_str(),
                    activity_scope_label(&item.scope),
                    activity_type,
                    queue
                ),
            );
        }
    }

    output
}

pub(crate) fn render_activity_queue_projection_json(
    projection: &ActivityQueueProjection,
) -> io::Result<String> {
    serde_json::to_string_pretty(projection).map_err(io::Error::other)
}

pub(crate) fn render_cost_inventory_text(projection: &CostInventoryProjection) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control Costs\n\n",
            "- Run: `{}`\n",
            "- Observations: total `{}`, run `{}`, step `{}`\n",
            "- Tokens: `{}`\n",
            "- Cost usd micros: `{}`\n",
            "- Latency ms: `{}` over `{}` observations\n"
        ),
        projection.run_id.as_str(),
        projection.summary.total,
        projection.summary.run_scoped,
        projection.summary.step_scoped,
        projection.summary.total_tokens,
        projection.summary.cost_usd_micros,
        projection.summary.latency_ms,
        projection.summary.latency_observations
    );

    if !projection.items.is_empty() {
        output.push_str("\n## Observations\n\n");
        for item in &projection.items {
            let model = item.observation.model.as_deref().unwrap_or("<none>");
            push_fmt(
                &mut output,
                format_args!(
                    "- seq `{}` [{}] provider `{}` model `{}` tokens `{}` cost_usd_micros `{}` latency_ms `{}`\n",
                    item.sequence,
                    activity_scope_label(&item.scope),
                    item.observation.provider,
                    model,
                    item.observation.observed_total_tokens(),
                    item.observation.cost_usd_micros,
                    item.observation
                        .latency_ms
                        .map_or_else(|| "<none>".to_owned(), |latency| latency.to_string())
                ),
            );
        }
    }

    output
}

pub(crate) fn render_cost_inventory_json(
    projection: &CostInventoryProjection,
) -> io::Result<String> {
    serde_json::to_string_pretty(projection).map_err(io::Error::other)
}
