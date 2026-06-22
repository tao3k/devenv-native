use super::context_path::{context_value_to_text, lookup_context_path};
#[cfg(feature = "wendao-integration")]
use super::query::resolve_dynamic_query_with_uri_expansion;
use super::wendao_uri::resolve_wendao_uri_with_zhenfa;
#[cfg(feature = "wendao-integration")]
use crate::workdir::{
    WorkdirSemanticScopeGuardStatus, WorkdirSemanticScopeGuardTrace,
    trace_workdir_semantic_scope_json, workdir_semantic_scope_guard_trace_json,
};
use serde_json::{Map, Value};

const SEMANTIC_SCOPE_METADATA_KEYS: &[&str] = &["semanticScopeMetadata", "semantic_scope_metadata"];
#[cfg(feature = "wendao-integration")]
const SEMANTIC_SCOPE_GUARD_TRACE_KEY: &str = "semanticScopeGuardTrace";
#[cfg(feature = "wendao-integration")]
const SEMANTIC_SCOPE_GUARD_ROUTE_KEY: &str = "semanticScopeGuardRoute";
const SEMANTIC_SCOPE_GUARD_POLICY_KEYS: &[&str] =
    &["semanticScopeGuardPolicy", "semantic_scope_guard_policy"];

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

#[cfg(feature = "wendao-integration")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticScopeGuardPolicy {
    Advisory,
    BlockOnBlocked,
    BlockOnReviewRequired,
}

#[cfg(feature = "wendao-integration")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticScopeGuardRecommendedAction {
    Continue,
    ReviewRequired,
    Blocked,
}

#[cfg(feature = "wendao-integration")]
impl SemanticScopeGuardPolicy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::BlockOnBlocked => "block_on_blocked",
            Self::BlockOnReviewRequired => "block_on_review_required",
        }
    }
}

#[cfg(feature = "wendao-integration")]
impl SemanticScopeGuardRecommendedAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::ReviewRequired => "review_required",
            Self::Blocked => "blocked",
        }
    }
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
            #[cfg(feature = "wendao-integration")]
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

#[cfg(feature = "wendao-integration")]
fn inject_semantic_scope_guard_trace(value: Value) -> Result<Value, String> {
    let Value::Object(mut object) = value else {
        return Ok(value);
    };
    let policy = semantic_scope_guard_policy(&object)?;
    let Some(raw_metadata_json) = semantic_scope_metadata_json(&object)? else {
        if policy != SemanticScopeGuardPolicy::Advisory {
            return Err(format!(
                "`semanticScopeGuardPolicy` `{}` requires `semanticScopeMetadata`",
                policy.as_str()
            ));
        }
        return Ok(Value::Object(object));
    };
    let trace = trace_workdir_semantic_scope_json(raw_metadata_json.as_str())
        .map_err(|error| format!("semantic-scope guard preflight failed: {error}"))?;
    enforce_semantic_scope_guard_policy(policy, &trace)?;
    object.insert(
        SEMANTIC_SCOPE_GUARD_TRACE_KEY.to_string(),
        workdir_semantic_scope_guard_trace_json(&trace),
    );
    object.insert(
        SEMANTIC_SCOPE_GUARD_ROUTE_KEY.to_string(),
        semantic_scope_guard_route_json(policy, &trace),
    );
    Ok(Value::Object(object))
}

#[cfg(not(feature = "wendao-integration"))]
fn inject_semantic_scope_guard_trace(value: Value) -> Result<Value, String> {
    let Value::Object(object) = &value else {
        return Ok(value);
    };
    let has_metadata = SEMANTIC_SCOPE_METADATA_KEYS
        .iter()
        .any(|key| object.contains_key(*key));
    let has_policy = SEMANTIC_SCOPE_GUARD_POLICY_KEYS
        .iter()
        .any(|key| object.contains_key(*key));
    if has_metadata || has_policy {
        return Err(
            "semantic-scope guard metadata requires the `wendao-integration` feature".to_string(),
        );
    }
    Ok(value)
}

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_route_json(
    policy: SemanticScopeGuardPolicy,
    trace: &WorkdirSemanticScopeGuardTrace,
) -> Value {
    let mut route = Map::with_capacity(4);
    route.insert(
        "policy".to_string(),
        Value::String(policy.as_str().to_string()),
    );
    route.insert(
        "status".to_string(),
        Value::String(semantic_scope_guard_status_token(trace.status).to_string()),
    );
    route.insert(
        "execution".to_string(),
        Value::String("continue".to_string()),
    );
    route.insert(
        "recommendedAction".to_string(),
        Value::String(
            semantic_scope_guard_recommended_action(trace.status)
                .as_str()
                .to_string(),
        ),
    );
    Value::Object(route)
}

#[cfg(feature = "wendao-integration")]
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

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_policy(
    object: &Map<String, Value>,
) -> Result<SemanticScopeGuardPolicy, String> {
    let Some((key, value)) = SEMANTIC_SCOPE_GUARD_POLICY_KEYS
        .iter()
        .find_map(|key| object.get(*key).map(|value| (*key, value)))
    else {
        return Ok(SemanticScopeGuardPolicy::Advisory);
    };

    match value {
        Value::Null => Ok(SemanticScopeGuardPolicy::Advisory),
        Value::String(raw) => semantic_scope_guard_policy_from_str(raw).ok_or_else(|| {
            format!(
                "`{key}` must be one of `advisory`, `block_on_blocked`, or `block_on_review_required`"
            )
        }),
        _ => Err(format!("`{key}` must be a string")),
    }
}

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_policy_from_str(raw: &str) -> Option<SemanticScopeGuardPolicy> {
    let normalized = raw.trim().replace('-', "_").to_ascii_lowercase();
    match normalized.as_str() {
        "" | "advisory" => Some(SemanticScopeGuardPolicy::Advisory),
        "block_on_blocked" | "blockonblocked" => Some(SemanticScopeGuardPolicy::BlockOnBlocked),
        "block_on_review_required" | "blockonreviewrequired" => {
            Some(SemanticScopeGuardPolicy::BlockOnReviewRequired)
        }
        _ => None,
    }
}

#[cfg(feature = "wendao-integration")]
fn enforce_semantic_scope_guard_policy(
    policy: SemanticScopeGuardPolicy,
    trace: &WorkdirSemanticScopeGuardTrace,
) -> Result<(), String> {
    match (policy, trace.status) {
        (SemanticScopeGuardPolicy::Advisory, _)
        | (
            SemanticScopeGuardPolicy::BlockOnBlocked
            | SemanticScopeGuardPolicy::BlockOnReviewRequired,
            WorkdirSemanticScopeGuardStatus::Ready,
        )
        | (
            SemanticScopeGuardPolicy::BlockOnBlocked,
            WorkdirSemanticScopeGuardStatus::ReviewRequired,
        ) => Ok(()),
        _ => Err(semantic_scope_guard_policy_error(policy, trace)),
    }
}

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_policy_error(
    policy: SemanticScopeGuardPolicy,
    trace: &WorkdirSemanticScopeGuardTrace,
) -> String {
    let issues = if trace.issues.is_empty() {
        "no semantic-scope issues reported".to_string()
    } else {
        trace.issues.join("; ")
    };
    format!(
        "semantic-scope guard policy `{}` blocked execution with status `{}`: {}",
        policy.as_str(),
        semantic_scope_guard_status_token(trace.status),
        issues
    )
}

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_status_token(status: WorkdirSemanticScopeGuardStatus) -> &'static str {
    match status {
        WorkdirSemanticScopeGuardStatus::Ready => "ready",
        WorkdirSemanticScopeGuardStatus::ReviewRequired => "review_required",
        WorkdirSemanticScopeGuardStatus::Blocked => "blocked",
    }
}

#[cfg(feature = "wendao-integration")]
fn semantic_scope_guard_recommended_action(
    status: WorkdirSemanticScopeGuardStatus,
) -> SemanticScopeGuardRecommendedAction {
    match status {
        WorkdirSemanticScopeGuardStatus::Ready => SemanticScopeGuardRecommendedAction::Continue,
        WorkdirSemanticScopeGuardStatus::ReviewRequired => {
            SemanticScopeGuardRecommendedAction::ReviewRequired
        }
        WorkdirSemanticScopeGuardStatus::Blocked => SemanticScopeGuardRecommendedAction::Blocked,
    }
}
