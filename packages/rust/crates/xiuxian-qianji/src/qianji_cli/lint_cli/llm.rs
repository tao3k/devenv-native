use std::fmt::Write as _;
use std::io;
use std::path::Path;

use ariadne::{CharSet, Config, IndexType, Label, LabelAttach, Report, ReportKind, Source};
use qianji_bpmn_engine::{LintIssue, LintReport};
use serde_json::Value;

use super::render::lint_domain_name;
use super::types::LintCliOutput;

pub(super) fn render_lint_llm_output(
    report: &LintReport,
    resolved_path: &Path,
    contents: Option<&str>,
) -> io::Result<LintCliOutput> {
    let exit_code = if report.ok { 0 } else { 2 };
    if report.ok {
        return Ok(LintCliOutput {
            rendered: format!(
                "[ok] {} {}\nno blocking issues found.\n",
                resolved_path.display(),
                lint_domain_name(&report.domain),
            ),
            exit_code,
        });
    }

    let mut rendered = format!(
        "[lint:error] {} {}\nSource: {}\nIssues: {}\n",
        resolved_path.display(),
        lint_domain_name(&report.domain),
        report.source_id,
        report.issues.len(),
    );

    for issue in &report.issues {
        if let (Some(contents), Some(_source_diagnostic)) =
            (contents, issue.source_diagnostic.as_ref())
        {
            rendered.push('\n');
            rendered.push_str(&render_ariadne_issue(issue, contents)?);
        } else {
            append_compact_issue(&mut rendered, issue);
        }
    }

    Ok(LintCliOutput {
        rendered,
        exit_code,
    })
}

fn render_ariadne_issue(issue: &LintIssue, contents: &str) -> io::Result<String> {
    let Some(diagnostic) = issue.source_diagnostic.as_ref() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source diagnostic is required for ariadne lint rendering",
        ));
    };
    let source_id = diagnostic.source_id.clone();
    let span = diagnostic.span.start..diagnostic.span.end;
    let mut output = Vec::new();

    Report::build(ReportKind::Error, (source_id.clone(), span.clone()))
        .with_config(
            Config::new()
                .with_color(false)
                .with_compact(true)
                .with_char_set(CharSet::Ascii)
                .with_index_type(IndexType::Byte)
                .with_label_attach(LabelAttach::Start)
                .with_cross_gap(false)
                .with_multiline_arrows(false),
        )
        .with_code(issue.code.as_str())
        .with_message(issue.title.as_str())
        .with_label(
            Label::new((source_id.clone(), span))
                .with_message(&diagnostic.label)
                .with_order(0),
        )
        .with_help(&diagnostic.help)
        .finish()
        .write((source_id, Source::from(contents)), &mut output)
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to render LLM lint diagnostic: {error}"),
            )
        })?;

    let mut rendered = String::from_utf8(output).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("LLM lint diagnostic was not UTF-8: {error}"),
        )
    })?;
    append_repair_block(&mut rendered, issue, Some(contents));
    Ok(rendered)
}

fn append_compact_issue(rendered: &mut String, issue: &LintIssue) {
    let _ = writeln!(rendered, "\n[error] {}", issue.code);
    let _ = writeln!(rendered, "{}", issue.title);
    let _ = writeln!(rendered, "{}", issue.summary);
    append_repair_block(rendered, issue, None);
}

fn append_repair_block(rendered: &mut String, issue: &LintIssue, contents: Option<&str>) {
    if append_proposed_patch(rendered, issue, contents) {
        return;
    }

    if let Some(structured_repair) = &issue.structured_repair
        && structured_expected_xml(structured_repair).is_some()
    {
        append_contract_message(rendered, issue);
        append_expected_xml(rendered, structured_repair);
        let _ = writeln!(rendered, "\nReturn unified diff only.");
        return;
    }

    let _ = writeln!(rendered, "\nAction:");
    let _ = writeln!(rendered, "- {}", issue.llm_fix_prompt);
    let _ = writeln!(rendered, "\nFix:");
    for step in &issue.repair_guidance {
        let _ = writeln!(rendered, "- {step}");
    }
    if let Some(structured_repair) = &issue.structured_repair {
        append_structured_repair_hints(rendered, structured_repair);
        append_expected_xml(rendered, structured_repair);
        let _ = writeln!(rendered, "\nStructured repair:");
        if let Some(strategy) = structured_repair
            .get("strategy")
            .and_then(|value| value.as_str())
        {
            let _ = writeln!(rendered, "- strategy: {strategy}");
        }
        if let Some(contract) = structured_repair
            .get("contract")
            .and_then(|value| value.as_str())
        {
            let _ = writeln!(rendered, "- contract: {contract}");
        }
    }
    if issue.structured_repair.is_none() && issue.evidence != serde_json::Value::Null {
        let _ = writeln!(rendered, "\nEvidence:");
        let evidence = serde_json::to_string_pretty(&issue.evidence)
            .unwrap_or_else(|_error| "{\"error\":\"failed to render lint evidence\"}".to_string());
        let _ = writeln!(rendered, "{evidence}");
    }
}

fn append_proposed_patch(rendered: &mut String, issue: &LintIssue, contents: Option<&str>) -> bool {
    let Some(line_fixes) = issue
        .structured_repair
        .as_ref()
        .and_then(|repair| repair.get("line_fixes"))
        .and_then(Value::as_array)
    else {
        return false;
    };
    if line_fixes.is_empty() {
        return false;
    }
    let Some(contents) = contents else {
        return false;
    };

    let Some(hunks) = proposed_patch_hunks(issue, contents, line_fixes) else {
        return false;
    };

    append_contract_message(rendered, issue);

    let _ = writeln!(rendered, "\nProposed patch:");
    let _ = writeln!(rendered, "```diff");
    if let Some(source_id) = issue
        .source_diagnostic
        .as_ref()
        .map(|diagnostic| diagnostic.source_id.as_str())
    {
        let _ = writeln!(rendered, "--- {source_id}");
        let _ = writeln!(rendered, "+++ {source_id}");
    }
    for hunk in hunks {
        let _ = writeln!(
            rendered,
            "@@ -{},1 +{},{} @@",
            hunk.line_number,
            hunk.line_number,
            hunk.replacements.len()
        );
        let _ = writeln!(rendered, "-{}", hunk.original);
        for replacement in hunk.replacements {
            let _ = writeln!(rendered, "+{replacement}");
        }
    }
    let _ = writeln!(rendered, "```");
    let _ = writeln!(rendered, "\nReturn unified diff only.");
    true
}

struct ProposedPatchHunk {
    line_number: usize,
    original: String,
    replacements: Vec<String>,
}

fn proposed_patch_hunks(
    issue: &LintIssue,
    contents: &str,
    line_fixes: &[Value],
) -> Option<Vec<ProposedPatchHunk>> {
    let mut hunks = Vec::new();
    for line_fix in line_fixes {
        let xml_lines = line_fix_xml_lines(line_fix)?;
        let offset = line_fix_offset(line_fix, issue)?;
        let (line_start, line_end) = line_bounds_for_offset(contents, offset)?;
        let original = contents.get(line_start..line_end)?.to_string();
        let line_number = line_number_for_offset(contents, offset);
        let indent = original
            .chars()
            .take_while(|character| character.is_whitespace())
            .collect::<String>();
        let replacements = xml_lines
            .into_iter()
            .map(|line| format!("{indent}{line}"))
            .collect::<Vec<_>>();
        hunks.push(ProposedPatchHunk {
            line_number,
            original,
            replacements,
        });
    }

    (!hunks.is_empty()).then_some(hunks)
}

fn line_fix_offset(line_fix: &Value, issue: &LintIssue) -> Option<usize> {
    line_fix
        .get("offset")
        .and_then(Value::as_u64)
        .and_then(|offset| usize::try_from(offset).ok())
        .or_else(|| {
            let line = line_fix.get("line")?;
            (line.as_str() == Some("primary"))
                .then(|| {
                    issue
                        .source_diagnostic
                        .as_ref()
                        .map(|diagnostic| diagnostic.span.start)
                })
                .flatten()
        })
}

fn line_bounds_for_offset(contents: &str, offset: usize) -> Option<(usize, usize)> {
    let bytes = contents.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let offset = offset.min(bytes.len().saturating_sub(1));
    let line_start = bytes[..=offset]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1);
    let line_end = bytes[offset..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |position| offset + position);
    Some((line_start, line_end))
}

fn line_number_for_offset(contents: &str, offset: usize) -> usize {
    let offset = offset.min(contents.len());
    contents.as_bytes()[..offset]
        .split(|byte| *byte == b'\n')
        .count()
}

fn line_fix_xml_lines(line_fix: &Value) -> Option<Vec<String>> {
    match line_fix.get("xml")? {
        Value::String(xml) => Some(xml.lines().map(ToString::to_string).collect()),
        Value::Array(lines) => {
            let xml_lines = lines
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            (!xml_lines.is_empty()).then_some(xml_lines)
        }
        _ => None,
    }
}

fn append_contract_message(rendered: &mut String, issue: &LintIssue) {
    if let Some(contract_message) = issue
        .structured_repair
        .as_ref()
        .and_then(|repair| repair.get("contract_message"))
        .and_then(Value::as_str)
    {
        let _ = writeln!(rendered, "    |Contract: {contract_message}");
    }
}

fn structured_expected_xml(structured_repair: &Value) -> Option<&str> {
    structured_repair
        .get("expected_xml")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn append_expected_xml(rendered: &mut String, structured_repair: &Value) {
    let Some(expected_xml) = structured_expected_xml(structured_repair) else {
        return;
    };
    let _ = writeln!(rendered, "\nExpected XML:");
    let _ = writeln!(rendered, "```xml");
    let _ = writeln!(rendered, "{expected_xml}");
    let _ = writeln!(rendered, "```");
}

fn append_structured_repair_hints(rendered: &mut String, structured_repair: &Value) {
    let Some(actions) = structured_repair.get("actions").and_then(Value::as_array) else {
        return;
    };

    let mut allowed_forms = Vec::new();
    let mut examples = Vec::new();
    let mut forbidden_forms = Vec::new();
    let mut operations = Vec::new();
    for action in actions {
        collect_allowed_forms(action.get("allowed_forms"), &mut allowed_forms);
        collect_nested_examples(action.get("allowed_forms"), &mut examples);
        collect_strings(action.get("examples"), &mut examples);
        collect_strings(action.get("forbidden_forms"), &mut forbidden_forms);
        collect_action_operations(action, &mut operations);
    }

    write_unique_list(rendered, "Patch focus", &operations);
    write_unique_list(rendered, "Allowed forms", &allowed_forms);
    write_unique_list(rendered, "Examples", &examples);
    write_unique_list(rendered, "Forbidden forms", &forbidden_forms);
}

fn collect_action_operations(action: &Value, operations: &mut Vec<String>) {
    if let Some(op) = action.get("op").and_then(Value::as_str) {
        operations.push(op.to_string());
    }
    if let Some(target) = action.get("target").and_then(Value::as_str) {
        operations.push(format!("target: {target}"));
    }
    for key in [
        "when",
        "requires",
        "also",
        "current",
        "preferred_default_flow_id",
        "preferred_default_has_condition",
        "valid_outgoing_flow_ids",
        "producer_change",
        "route_change",
        "forbid",
    ] {
        collect_named_repair_value(action, key, operations);
    }
    if let Some(options) = action.get("options").and_then(Value::as_array) {
        for option in options {
            collect_action_operations(option, operations);
        }
    }
}

fn collect_named_repair_value(action: &Value, key: &str, operations: &mut Vec<String>) {
    let Some(value) = action.get(key) else {
        return;
    };
    match value {
        Value::String(value) => operations.push(format!("{key}: {value}")),
        Value::Array(values) => {
            let values = values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            if !values.is_empty() {
                operations.push(format!("{key}: {values}"));
            }
        }
        Value::Bool(value) => operations.push(format!("{key}: {value}")),
        _ => {}
    }
}

fn collect_allowed_forms(value: Option<&Value>, output: &mut Vec<String>) {
    match value {
        Some(Value::Object(forms)) => {
            for (name, shape) in forms {
                output.push(name.clone());
                collect_strings(Some(shape), output);
            }
        }
        other => collect_strings(other, output),
    }
}

fn collect_nested_examples(value: Option<&Value>, output: &mut Vec<String>) {
    match value {
        Some(Value::Object(object)) => {
            if let Some(examples) = object.get("examples") {
                collect_strings(Some(examples), output);
            }
            for value in object.values() {
                collect_nested_examples(Some(value), output);
            }
        }
        Some(Value::Array(values)) => {
            for value in values {
                collect_nested_examples(Some(value), output);
            }
        }
        _ => {}
    }
}

fn collect_strings(value: Option<&Value>, output: &mut Vec<String>) {
    match value {
        Some(Value::String(value)) => output.push(value.clone()),
        Some(Value::Array(values)) => {
            for value in values {
                collect_strings(Some(value), output);
            }
        }
        _ => {}
    }
}

fn write_unique_list(rendered: &mut String, title: &str, values: &[String]) {
    let mut seen = std::collections::BTreeSet::new();
    let values = values
        .iter()
        .filter(|value| seen.insert(value.as_str()))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }
    let _ = writeln!(rendered, "\n{title}:");
    for value in values {
        let _ = writeln!(rendered, "- {value}");
    }
}
