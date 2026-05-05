use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
};

use crate::studio::types::{AutocompleteResponse, AutocompleteSuggestion};

pub(super) fn autocomplete_suggestion_batch(
    suggestions: &[AutocompleteSuggestion],
) -> Result<LanceRecordBatch, String> {
    let texts = suggestions
        .iter()
        .map(|suggestion| suggestion.text.as_str())
        .collect::<Vec<_>>();
    let suggestion_types = suggestions
        .iter()
        .map(|suggestion| suggestion.suggestion_type.as_str())
        .collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("text", LanceDataType::Utf8, false),
            LanceField::new("suggestionType", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(texts)),
            Arc::new(LanceStringArray::from(suggestion_types)),
        ],
    )
    .map_err(|error| format!("failed to build autocomplete Flight batch: {error}"))
}

pub(super) fn autocomplete_response_flight_app_metadata(
    response: &AutocompleteResponse,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "prefix": response.prefix,
    }))
    .map_err(|error| format!("failed to encode autocomplete Flight app metadata: {error}"))
}
