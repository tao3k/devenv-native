//! Semantic-scope analysis route contract and metadata validation.

/// Stable route for the semantic-scope analysis contract.
pub const ANALYSIS_SEMANTIC_SCOPE_ROUTE: &str = "/analysis/semantic-scope";
/// Optional task ID metadata header for semantic-scope requests.
pub const WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER: &str = "x-wendao-semantic-task-id";
/// Optional comma-separated object ID metadata header for semantic-scope requests.
pub const WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER: &str = "x-wendao-semantic-object-ids";

/// Transport-owned semantic-scope request decoded from metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticScopeFlightRequest {
    /// Optional task object ID anchoring the semantic scope.
    pub task_id: Option<String>,
    /// Optional explicit object IDs anchoring the semantic scope.
    pub object_ids: Vec<String>,
}

/// Validate the stable semantic-scope analysis request contract.
///
/// # Errors
///
/// Returns an error when a task ID or object ID is present but blank.
pub fn validate_semantic_scope_request(
    task_id: Option<&str>,
    object_ids: &[String],
) -> Result<SemanticScopeFlightRequest, String> {
    if matches!(task_id, Some(value) if value.trim().is_empty()) {
        return Err("semantic-scope task id must not be blank".to_string());
    }
    let task_id = task_id.map(str::trim).map(ToString::to_string);

    let mut normalized_object_ids = Vec::new();
    for object_id in object_ids {
        let trimmed = object_id.trim();
        if trimmed.is_empty() {
            return Err("semantic-scope object ids must not contain blanks".to_string());
        }
        normalized_object_ids.push(trimmed.to_string());
    }

    Ok(SemanticScopeFlightRequest {
        task_id,
        object_ids: normalized_object_ids,
    })
}
