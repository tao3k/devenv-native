use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
    LanceUInt64Array,
};

use crate::gateway::studio::types::SymbolSearchHit;

pub(super) fn build_symbol_hits_flight_batch(
    hits: &[SymbolSearchHit],
) -> Result<LanceRecordBatch, String> {
    let names = hits.iter().map(|hit| hit.name.as_str()).collect::<Vec<_>>();
    let kinds = hits.iter().map(|hit| hit.kind.as_str()).collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.as_str()).collect::<Vec<_>>();
    let lines = hits.iter().map(|hit| hit.line as u64).collect::<Vec<_>>();
    let locations = hits
        .iter()
        .map(|hit| hit.location.as_str())
        .collect::<Vec<_>>();
    let languages = hits
        .iter()
        .map(|hit| hit.language.as_str())
        .collect::<Vec<_>>();
    let sources = hits
        .iter()
        .map(|hit| hit.source.as_str())
        .collect::<Vec<_>>();
    let crate_names = hits
        .iter()
        .map(|hit| hit.crate_name.as_str())
        .collect::<Vec<_>>();
    let project_names = hits
        .iter()
        .map(|hit| hit.project_name.as_deref())
        .collect::<Vec<_>>();
    let root_labels = hits
        .iter()
        .map(|hit| hit.root_label.as_deref())
        .collect::<Vec<_>>();
    let navigation_targets_json = hits
        .iter()
        .map(|hit| serde_json::to_string(&hit.navigation_target).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let scores = hits.iter().map(|hit| hit.score).collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("name", LanceDataType::Utf8, false),
            LanceField::new("kind", LanceDataType::Utf8, false),
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("line", LanceDataType::UInt64, false),
            LanceField::new("location", LanceDataType::Utf8, false),
            LanceField::new("language", LanceDataType::Utf8, false),
            LanceField::new("source", LanceDataType::Utf8, false),
            LanceField::new("crateName", LanceDataType::Utf8, false),
            LanceField::new("projectName", LanceDataType::Utf8, true),
            LanceField::new("rootLabel", LanceDataType::Utf8, true),
            LanceField::new("navigationTargetJson", LanceDataType::Utf8, false),
            LanceField::new("score", LanceDataType::Float64, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(kinds)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceUInt64Array::from(lines)),
            Arc::new(LanceStringArray::from(locations)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceStringArray::from(sources)),
            Arc::new(LanceStringArray::from(crate_names)),
            Arc::new(LanceStringArray::from(project_names)),
            Arc::new(LanceStringArray::from(root_labels)),
            Arc::new(LanceStringArray::from(
                navigation_targets_json
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceFloat64Array::from(scores)),
        ],
    )
    .map_err(|error| format!("failed to build symbol-search Flight batch: {error}"))
}
