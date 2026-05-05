use super::context_path::{context_value_to_text, lookup_context_path};
use super::query::resolve_dynamic_query_with_uri_expansion;
use super::wendao_uri::resolve_wendao_uri_with_zhenfa;
use crate::workdir::{trace_workdir_semantic_scope_json, workdir_semantic_scope_guard_trace_json};
use serde_json::{Map, Value};

const SEMANTIC_SCOPE_METADATA_KEYS: &[&str] = &["semanticScopeMetadata", "semantic_scope_metadata"];
const SEMANTIC_SCOPE_GUARD_TRACE_KEY: &str = "semanticScopeGuardTrace";

/// Resolves `$wendao://...` placeholders recursively before node execution.
///
/// # Errors
///
/// Returns an error when a placeholder token is empty or when one semantic URI
/// cannot be resolved from embedded Wendao resources.
pub(crate) fn resolve_wendao_placeholders_in_context(context: &Value) -> Result<Value, String> {
    let resolved = resolve_value(context, context)?;
    inject_semantic_scope_guard_trace(resolved)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticResolutionMode {
    Content,
    Reference,
}

fn resolve_value(value: &Value, context: &Value) -> Result<Value, String> {
    match value {
        Value::String(raw) => {
            resolve_string(raw, context, SemanticResolutionMode::Content).map(Value::String)
        }
        Value::Array(items) => items
            .iter()
            .map(|item| resolve_value(item, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(object) => {
            let mut resolved = Map::with_capacity(object.len());
            for (key, item) in object {
                resolved.insert(key.clone(), resolve_value(item, context)?);
            }
            Ok(Value::Object(resolved))
        }
        _ => Ok(value.clone()),
    }
}

fn resolve_string(
    raw: &str,
    context: &Value,
    mode: SemanticResolutionMode,
) -> Result<String, String> {
    let trimmed = raw.trim();
    let Some(token) = trimmed.strip_prefix('$') else {
        return match mode {
            SemanticResolutionMode::Content => Ok(raw.to_string()),
            SemanticResolutionMode::Reference => Ok(trimmed.to_string()),
        };
    };
    let token = token.trim();
    if token.is_empty() {
        return Err("semantic placeholder must not be empty".to_string());
    }

    if token.starts_with("wendao://") {
        return match mode {
            SemanticResolutionMode::Content => resolve_wendao_uri_with_zhenfa(token),
            SemanticResolutionMode::Reference => Ok(token.to_string()),
        };
    }

    if let Some(value) = lookup_context_path(context, token)
        && let Some(text) = context_value_to_text(value)
    {
        return Ok(text);
    }

    match mode {
        SemanticResolutionMode::Content => {
            if let Some(expanded) = resolve_dynamic_query_with_uri_expansion(token)? {
                return Ok(expanded);
            }
            Ok(raw.to_string())
        }
        SemanticResolutionMode::Reference => Ok(token.to_string()),
    }
}

/// Resolves a semantic placeholder (`$...`) as runtime content.
///
/// Resolution order:
/// 1. `$wendao://...` -> embedded semantic resource payload.
/// 2. `$context.path` -> current context value text.
/// 3. `$<query>` -> dynamic Wendao URI expansion XML-Lite.
/// 4. unresolved -> original raw input.
///
/// # Errors
///
/// Returns an error when the placeholder token is empty or when semantic
/// resource/query resolution fails.
pub(crate) fn resolve_semantic_content(raw: &str, context: &Value) -> Result<String, String> {
    resolve_string(raw, context, SemanticResolutionMode::Content)
}

/// Resolves a semantic placeholder (`$...`) as one symbolic reference value.
///
/// Resolution order:
/// 1. `$context.path` -> current context value text.
/// 2. `$wendao://...` -> canonical URI string (no dereference).
/// 3. unresolved -> token text (without `$`).
///
/// # Errors
///
/// Returns an error when the placeholder token is empty.
pub(crate) fn resolve_semantic_reference(raw: &str, context: &Value) -> Result<String, String> {
    resolve_string(raw, context, SemanticResolutionMode::Reference)
}

fn inject_semantic_scope_guard_trace(value: Value) -> Result<Value, String> {
    let Value::Object(mut object) = value else {
        return Ok(value);
    };
    let Some(raw_metadata_json) = semantic_scope_metadata_json(&object)? else {
        return Ok(Value::Object(object));
    };
    let trace = trace_workdir_semantic_scope_json(raw_metadata_json.as_str())
        .map_err(|error| format!("semantic-scope guard preflight failed: {error}"))?;
    object.insert(
        SEMANTIC_SCOPE_GUARD_TRACE_KEY.to_string(),
        workdir_semantic_scope_guard_trace_json(&trace),
    );
    Ok(Value::Object(object))
}

fn semantic_scope_metadata_json(object: &Map<String, Value>) -> Result<Option<String>, String> {
    let Some((key, value)) = SEMANTIC_SCOPE_METADATA_KEYS
        .iter()
        .find_map(|key| object.get(*key).map(|value| (*key, value)))
    else {
        return Ok(None);
    };

    match value {
        Value::Null => Ok(None),
        Value::String(raw) => Ok(Some(raw.clone())),
        Value::Object(_) => serde_json::to_string(value)
            .map(Some)
            .map_err(|error| format!("failed to encode `{key}` semantic-scope metadata: {error}")),
        _ => Err(format!("`{key}` must be a JSON object or JSON string")),
    }
}
