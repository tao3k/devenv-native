use super::macros::define_native_tool;
use serde_json::json;
use std::sync::Arc;
use xiuxian_zhixing::ZhixingHeyi;

fn reminder_recipient_from_session_id(session_id: Option<&str>) -> Option<String> {
    let raw = session_id?;
    let (channel, key) = raw.split_once(':')?;
    let channel = channel.trim();
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    match channel {
        "telegram" => Some(format!("telegram:{key}")),
        "discord" => Some(format!("discord:{key}")),
        _ => None,
    }
}

define_native_tool! {
    /// Native tool for recording journal entries.
    pub struct JournalRecordTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "journal.record",
    description: "Record a daily journal entry or reflection. Use this to compile raw thoughts into structured insights.",
    parameters: json!({
            "type": "object",
            "properties": {
                "content": { "type": "string", "description": "The unstructured journal content" }
            },
            "required": ["content"]
        }),
    call(|tool, arguments, _context| {
        let content = arguments
            .and_then(|a| a["content"].as_str().map(ToString::to_string))
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' argument"))?;

        let mut journal = xiuxian_zhixing::journal::JournalEntry::new(content);
        let insight = tool.heyi.reflect(&mut journal).await?;
        Ok(format!("Journal recorded. Insight: {insight}"))
    })
}

define_native_tool! {
    /// Native tool for adding a specific task to the agenda.
    pub struct TaskAddTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "task.add",
    description: "Add a new task or 'Vow' to your cultivation agenda. If the user specifies a time (e.g. 'Watch movie at 7pm'), you MUST parse it into the user's local timezone as an RFC3339 string and populate 'scheduled_at'.",
    parameters: json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "The title or description of the task" },
                "scheduled_at": { "type": "string", "description": "Optional: The scheduled time for the task in RFC3339 format (e.g., '2026-02-25T19:00:00-08:00')." }
            },
            "required": ["title"]
        }),
    call(|tool, arguments, context| {
        let title = arguments
            .as_ref()
            .and_then(|a| a["title"].as_str().map(ToString::to_string))
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' argument"))?;

        let scheduled_at = arguments
            .as_ref()
            .and_then(|a| {
                a.get("scheduled_at")
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| a.get("time").and_then(serde_json::Value::as_str))
            })
            .map(str::trim)
            .filter(|raw| !raw.is_empty())
            .map(|raw| tool.heyi.normalize_scheduled_time_input(raw))
            .transpose()?;
        let reminder_recipient =
            reminder_recipient_from_session_id(context.session_id.as_deref());

        let result = tool
            .heyi
            .add_task_with_recipient(&title, scheduled_at, reminder_recipient)
            .await?;
        Ok(result)
    })
}

define_native_tool! {
    /// Native tool for viewing the agenda.
    pub struct AgendaViewTool {
        /// Reference to the Heyi orchestrator.
        pub heyi: Arc<ZhixingHeyi>,
    }
    name: "agenda.view",
    description: "View the current cultivation agenda, including active vows and critically stale tasks.",
    parameters: json!({ "type": "object", "properties": {} }),
    call(|tool, _args, _context| {
        tool.heyi.render_agenda().map_err(|e| anyhow::anyhow!(e))
    })
}
