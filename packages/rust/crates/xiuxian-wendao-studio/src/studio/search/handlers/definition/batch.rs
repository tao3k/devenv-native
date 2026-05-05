use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceFloat64Array, LanceInt32Array, LanceRecordBatch, LanceSchema,
    LanceStringArray,
};

use crate::studio::types::{DefinitionResolveResponse, DefinitionSearchHit};

pub(super) fn definition_hit_batch(hit: &DefinitionSearchHit) -> Result<LanceRecordBatch, String> {
    let observation_hints_json = serde_json::to_string(&hit.observation_hints)
        .map_err(|error| format!("failed to encode definition observation hints: {error}"))?;
    let navigation_target_json = serde_json::to_string(&hit.navigation_target)
        .map_err(|error| format!("failed to encode definition navigation target: {error}"))?;

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("name", LanceDataType::Utf8, false),
            LanceField::new("signature", LanceDataType::Utf8, false),
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("language", LanceDataType::Utf8, false),
            LanceField::new("crateName", LanceDataType::Utf8, false),
            LanceField::new("projectName", LanceDataType::Utf8, true),
            LanceField::new("rootLabel", LanceDataType::Utf8, true),
            LanceField::new("nodeKind", LanceDataType::Utf8, true),
            LanceField::new("ownerTitle", LanceDataType::Utf8, true),
            LanceField::new("navigationTargetJson", LanceDataType::Utf8, false),
            LanceField::new("lineStart", LanceDataType::Int32, false),
            LanceField::new("lineEnd", LanceDataType::Int32, false),
            LanceField::new("score", LanceDataType::Float64, false),
            LanceField::new("observationHintsJson", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![hit.name.as_str()])),
            Arc::new(LanceStringArray::from(vec![hit.signature.as_str()])),
            Arc::new(LanceStringArray::from(vec![hit.path.as_str()])),
            Arc::new(LanceStringArray::from(vec![hit.language.as_str()])),
            Arc::new(LanceStringArray::from(vec![hit.crate_name.as_str()])),
            Arc::new(LanceStringArray::from(vec![hit.project_name.as_deref()])),
            Arc::new(LanceStringArray::from(vec![hit.root_label.as_deref()])),
            Arc::new(LanceStringArray::from(vec![hit.node_kind.as_deref()])),
            Arc::new(LanceStringArray::from(vec![hit.owner_title.as_deref()])),
            Arc::new(LanceStringArray::from(vec![
                navigation_target_json.as_str(),
            ])),
            Arc::new(LanceInt32Array::from(vec![
                i32::try_from(hit.line_start).map_err(|error| {
                    format!("failed to represent definition line_start: {error}")
                })?,
            ])),
            Arc::new(LanceInt32Array::from(vec![
                i32::try_from(hit.line_end)
                    .map_err(|error| format!("failed to represent definition line_end: {error}"))?,
            ])),
            Arc::new(LanceFloat64Array::from(vec![hit.score])),
            Arc::new(LanceStringArray::from(vec![
                observation_hints_json.as_str(),
            ])),
        ],
    )
    .map_err(|error| format!("failed to build definition Flight batch: {error}"))
}

pub(super) fn definition_response_flight_app_metadata(
    response: &DefinitionResolveResponse,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "query": response.query,
        "sourcePath": response.source_path,
        "sourceLine": response.source_line,
        "candidateCount": response.candidate_count,
        "selectedScope": response.selected_scope,
        "navigationTarget": response.navigation_target,
        "resolvedTarget": response.resolved_target,
    }))
    .map_err(|error| format!("failed to encode definition Flight app metadata: {error}"))
}
