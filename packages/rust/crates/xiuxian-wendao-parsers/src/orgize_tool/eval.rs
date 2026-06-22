//! Org Babel eval contract adapter.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use orgize::Org;
use orgize::ast::{
    BabelEvalOutput, BabelEvalPlan, BabelEvalResultPatch, BabelEvalResultPatchKind,
    SourceBlockEvalPolicy, SourceBlockHeaderArg, SourceBlockHeaderArgSource,
    SourceBlockResultHandling, SourceBlockResultValueType,
};

use super::OrgizeToolError;
use super::io::read_to_string;

/// Request for rendering an Org Babel eval plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeEvalPlanRequest {
    /// Named `#+NAME:` source block to resolve.
    pub name: String,
    /// Org file containing the named source block.
    pub path: PathBuf,
    /// Render machine-readable JSON instead of compact text.
    pub json: bool,
}

/// Request for rendering or applying an Org Babel result patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeEvalPatchRequest {
    /// Named `#+NAME:` source block to resolve.
    pub name: String,
    /// Org file containing the named source block.
    pub path: PathBuf,
    /// Host-supplied stdout.
    pub stdout: String,
    /// Host-supplied stderr.
    pub stderr: String,
    /// Host-supplied process exit code.
    pub exit_code: Option<i32>,
    /// Write the rendered `#+RESULTS:` patch to the Org file.
    pub write: bool,
    /// Render machine-readable JSON instead of compact text.
    pub json: bool,
}

/// Renders the parser-owned eval contract for one named Org Babel source block.
///
/// # Errors
///
/// Returns an error when the file cannot be read or the named source block
/// cannot be resolved uniquely.
pub fn render_eval_plan(request: &OrgizeEvalPlanRequest) -> Result<String, OrgizeToolError> {
    let source = read_to_string(&request.path)?;
    let plan = eval_plan_from_source(&source, &request.name)?;
    Ok(if request.json {
        format!(
            "{}\n",
            eval_plan_json(&plan, &request.path.display().to_string())
        )
    } else {
        eval_plan_compact(&plan)
    })
}

/// Renders or applies the `#+RESULTS:` patch for one named Org Babel source block.
///
/// # Errors
///
/// Returns an error when the file cannot be read or written, or the named source
/// block cannot be resolved uniquely.
pub fn render_eval_patch(request: &OrgizeEvalPatchRequest) -> Result<String, OrgizeToolError> {
    let source = read_to_string(&request.path)?;
    let plan = eval_plan_from_source(&source, &request.name)?;
    let output = BabelEvalOutput {
        stdout: request.stdout.clone(),
        stderr: request.stderr.clone(),
        exit_code: request.exit_code,
    };
    let patch = plan.result_patch(&source, &output);
    if request.write {
        let next = patch.apply_to(&source);
        if next != source {
            fs::write(&request.path, next).map_err(|source| OrgizeToolError::Io {
                path: request.path.clone(),
                source,
            })?;
        }
    }
    Ok(if request.json {
        format!(
            "{}\n",
            eval_patch_json(
                &plan,
                &patch,
                &request.path.display().to_string(),
                request.write
            )
        )
    } else {
        eval_patch_compact(&plan, &patch, request.write)
    })
}

fn eval_plan_from_source(source: &str, name: &str) -> Result<BabelEvalPlan, OrgizeToolError> {
    Org::parse(source)
        .document()
        .babel_eval_plan(name)
        .map_err(|reason| OrgizeToolError::EvalPlan {
            name: name.to_string(),
            reason,
        })
}

fn eval_plan_compact(plan: &BabelEvalPlan) -> String {
    let record = &plan.record;
    let mut rendered = String::new();
    let _ = writeln!(rendered, "name: {}", plan.name);
    if let Some(language) = record.language.as_deref() {
        let _ = writeln!(rendered, "language: {language}");
    }
    let _ = writeln!(
        rendered,
        "eval: {}",
        eval_policy_label(record.execution.eval.policy)
    );
    let _ = writeln!(
        rendered,
        "results: {} {}",
        result_value_type_label(record.result_options.value_type),
        result_handling_label(record.result_options.handling)
    );
    let _ = writeln!(
        rendered,
        "block: {}..{}",
        record.source.range_start, record.source.range_end
    );
    if let Some(result) = &record.result {
        let _ = writeln!(
            rendered,
            "result: {}..{}",
            result.source.range_start, result.source.range_end
        );
    } else {
        rendered.push_str("result: none\n");
    }
    rendered
}

fn eval_patch_compact(plan: &BabelEvalPlan, patch: &BabelEvalResultPatch, written: bool) -> String {
    let mut rendered = String::new();
    let _ = writeln!(rendered, "name: {}", plan.name);
    let _ = writeln!(rendered, "kind: {}", patch_kind_label(patch.kind));
    let _ = writeln!(
        rendered,
        "handling: {}",
        result_handling_label(patch.handling)
    );
    if let Some(range) = patch.range {
        let _ = writeln!(rendered, "result: {}..{}", range.start, range.end);
    } else {
        rendered.push_str("result: none\n");
    }
    let _ = writeln!(rendered, "written: {written}");
    if let Some(message) = patch.message.as_deref() {
        let _ = writeln!(rendered, "message: {message}");
    }
    if !written && !patch.replacement.is_empty() {
        rendered.push_str("replacement:\n");
        rendered.push_str(&patch.replacement);
    }
    rendered
}

fn eval_plan_json(plan: &BabelEvalPlan, source_path: &str) -> serde_json::Value {
    let record = &plan.record;
    serde_json::json!({
        "source": source_path,
        "name": plan.name,
        "language": record.language,
        "parameters": record.parameters,
        "body": record.value,
        "eval": {
            "raw": record.execution.eval.raw,
            "policy": eval_policy_label(record.execution.eval.policy),
            "source": header_source_label(record.execution.eval.source),
        },
        "results": {
            "raw": record.result_options.raw,
            "handling": result_handling_label(record.result_options.handling),
            "valueType": result_value_type_label(record.result_options.value_type),
            "tokens": record.result_options.tokens,
        },
        "headerArgs": header_args_json(&record.normalized_header_args),
        "blockRange": {
            "start": record.source.range_start,
            "end": record.source.range_end,
        },
        "resultRange": record.result.as_ref().map(|result| serde_json::json!({
            "start": result.source.range_start,
            "end": result.source.range_end,
        })),
    })
}

fn eval_patch_json(
    plan: &BabelEvalPlan,
    patch: &BabelEvalResultPatch,
    source_path: &str,
    written: bool,
) -> serde_json::Value {
    serde_json::json!({
        "source": source_path,
        "name": plan.name,
        "kind": patch_kind_label(patch.kind),
        "handling": result_handling_label(patch.handling),
        "range": patch.range.map(|range| serde_json::json!({
            "start": range.start,
            "end": range.end,
        })),
        "replacement": patch.replacement,
        "written": written,
        "message": patch.message,
    })
}

fn header_args_json(args: &[SourceBlockHeaderArg]) -> Vec<serde_json::Value> {
    args.iter()
        .map(|arg| {
            serde_json::json!({
                "key": arg.key,
                "value": arg.value,
                "raw": arg.raw,
                "source": header_source_label(arg.source),
                "tokens": arg.tokens,
            })
        })
        .collect()
}

fn eval_policy_label(policy: SourceBlockEvalPolicy) -> &'static str {
    match policy {
        SourceBlockEvalPolicy::Yes => "yes",
        SourceBlockEvalPolicy::No => "no",
        SourceBlockEvalPolicy::NoExport => "no-export",
        SourceBlockEvalPolicy::StripExport => "strip-export",
        SourceBlockEvalPolicy::NeverExport => "never-export",
        SourceBlockEvalPolicy::Eval => "eval",
        SourceBlockEvalPolicy::Never => "never",
        SourceBlockEvalPolicy::Query => "query",
        SourceBlockEvalPolicy::Other => "other",
    }
}

fn result_handling_label(handling: SourceBlockResultHandling) -> &'static str {
    match handling {
        SourceBlockResultHandling::Replace => "replace",
        SourceBlockResultHandling::Silent => "silent",
        SourceBlockResultHandling::None => "none",
        SourceBlockResultHandling::Discard => "discard",
        SourceBlockResultHandling::Append => "append",
        SourceBlockResultHandling::Prepend => "prepend",
    }
}

fn result_value_type_label(value_type: SourceBlockResultValueType) -> &'static str {
    match value_type {
        SourceBlockResultValueType::Value => "value",
        SourceBlockResultValueType::Output => "output",
    }
}

fn header_source_label(source: SourceBlockHeaderArgSource) -> &'static str {
    match source {
        SourceBlockHeaderArgSource::Explicit => "explicit",
        SourceBlockHeaderArgSource::Default => "default",
    }
}

fn patch_kind_label(kind: BabelEvalResultPatchKind) -> &'static str {
    match kind {
        BabelEvalResultPatchKind::Insert => "insert",
        BabelEvalResultPatchKind::Replace => "replace",
        BabelEvalResultPatchKind::Noop => "noop",
    }
}
