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

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_SEMANTIC_SCOPE_ROUTE, WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
        WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER, validate_semantic_scope_request,
    };

    #[test]
    fn semantic_scope_contract_exposes_stable_route_and_headers() {
        assert_eq!(ANALYSIS_SEMANTIC_SCOPE_ROUTE, "/analysis/semantic-scope");
        assert_eq!(
            WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
            "x-wendao-semantic-task-id"
        );
        assert_eq!(
            WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
            "x-wendao-semantic-object-ids"
        );
    }

    #[test]
    fn semantic_scope_request_allows_default_active_scope() {
        let request = validate_semantic_scope_request(None, &[])
            .unwrap_or_else(|error| panic!("default semantic scope should validate: {error}"));

        assert!(request.task_id.is_none());
        assert!(request.object_ids.is_empty());
    }

    #[test]
    fn semantic_scope_request_rejects_blank_object_ids() {
        let result = validate_semantic_scope_request(None, &[String::new()]);

        assert!(result.is_err());
    }
}
