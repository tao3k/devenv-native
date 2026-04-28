use super::{ConstructCard, construct_index_entries};

/// Render a compact construct-card table of contents.
#[must_use]
pub(crate) fn render_construct_index(cards: &[ConstructCard]) -> String {
    let mut lines = vec![
        "# Qianji Construct Index".to_string(),
        String::new(),
        "Use this as a table of contents after reading the source task or `SKILL.md`. The source file is semantic input, not automatically a workflow artifact.".to_string(),
        String::new(),
        "First decide the scenario shape from the source: autonomous workflow, interactive workflow, or planning workflow that must ask the user before execution. Then select only the cards needed for that scenario and run `qianji construct show <id>` for details.".to_string(),
        String::new(),
        "Scenario hints: choose interactive when the source asks a human/user/partner for approval, answers to subagent questions, missing context, choices, reviews, or escalation handling. Choose autonomous only when every decision and context answer can come from host task outputs without human input.".to_string(),
        String::new(),
        "| ID | Domain | Status | Summary |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];
    for card in cards {
        lines.push(format!(
            "| `{}` | {} | {} | {} |",
            card.id,
            card.domain,
            card.status.as_str(),
            card.summary
        ));
    }
    lines.push(String::new());
    lines.push("Suggested LLM flow: read source skill/task -> classify autonomous vs interactive vs planning scenario -> pick construct ids -> inspect cards -> fill a BPMN or DMN scaffold -> run `qianji lint`.".to_string());
    lines.join("\n")
}

/// Render the construct index as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static catalog cannot be serialized.
pub(crate) fn render_construct_index_json(cards: &[ConstructCard]) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&construct_index_entries(cards))
}

/// Render one detailed construct card.
#[must_use]
pub(crate) fn render_construct_card(card: &ConstructCard) -> String {
    let mut lines = vec![
        format!("# Qianji Construct Card: {}", card.id),
        String::new(),
        format!("Title: {}", card.title),
        format!("Domain: {}", card.domain),
        format!("Status: {}", card.status.as_str()),
        format!("Summary: {}", card.summary),
        String::new(),
        "## Purpose".to_string(),
        String::new(),
        card.purpose.to_string(),
        String::new(),
        "## Requires".to_string(),
        String::new(),
    ];
    push_bullets(&mut lines, card.requires);
    lines.extend([String::new(), "## Allows".to_string(), String::new()]);
    push_bullets(&mut lines, card.allows);
    lines.extend([String::new(), "## Forbids".to_string(), String::new()]);
    push_bullets(&mut lines, card.forbids);
    lines.extend([
        String::new(),
        "## Example".to_string(),
        String::new(),
        "```xml".to_string(),
        card.example.to_string(),
        "```".to_string(),
        String::new(),
        "## Lint Repair Map".to_string(),
        String::new(),
    ]);
    for mapping in card.lint_mappings {
        lines.push(format!("- `{}`: {}", mapping.diagnostic, mapping.repair));
    }
    lines.extend([String::new(), "## Related Cards".to_string(), String::new()]);
    push_bullets(&mut lines, card.next_cards);
    lines.join("\n")
}

/// Render one detailed construct card as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static construct card cannot be serialized.
pub(crate) fn render_construct_card_json(card: &ConstructCard) -> serde_json::Result<String> {
    serde_json::to_string_pretty(card)
}

fn push_bullets(lines: &mut Vec<String>, values: &[&str]) {
    if values.is_empty() {
        lines.push("- none".to_string());
        return;
    }
    for value in values {
        lines.push(format!("- {value}"));
    }
}
