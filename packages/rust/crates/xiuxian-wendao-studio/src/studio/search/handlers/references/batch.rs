use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
    LanceUInt64Array,
};

use crate::studio::types::ReferenceSearchHit;

pub(super) fn build_reference_hits_flight_batch(
    hits: &[ReferenceSearchHit],
) -> Result<LanceRecordBatch, String> {
    let names = hits.iter().map(|hit| hit.name.clone()).collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>();
    let languages = hits
        .iter()
        .map(|hit| hit.language.clone())
        .collect::<Vec<_>>();
    let crate_names = hits
        .iter()
        .map(|hit| hit.crate_name.clone())
        .collect::<Vec<_>>();
    let project_names = hits
        .iter()
        .map(|hit| hit.project_name.clone().unwrap_or_default())
        .collect::<Vec<_>>();
    let root_labels = hits
        .iter()
        .map(|hit| hit.root_label.clone().unwrap_or_default())
        .collect::<Vec<_>>();
    let navigation_targets_json = hits
        .iter()
        .map(|hit| serde_json::to_string(&hit.navigation_target).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let lines = hits.iter().map(|hit| hit.line as u64).collect::<Vec<_>>();
    let columns = hits.iter().map(|hit| hit.column as u64).collect::<Vec<_>>();
    let line_texts = hits
        .iter()
        .map(|hit| hit.line_text.clone())
        .collect::<Vec<_>>();
    let scores = hits.iter().map(|hit| hit.score).collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("name", LanceDataType::Utf8, false),
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("language", LanceDataType::Utf8, false),
            LanceField::new("crateName", LanceDataType::Utf8, false),
            LanceField::new("projectName", LanceDataType::Utf8, false),
            LanceField::new("rootLabel", LanceDataType::Utf8, false),
            LanceField::new("navigationTargetJson", LanceDataType::Utf8, false),
            LanceField::new("line", LanceDataType::UInt64, false),
            LanceField::new("column", LanceDataType::UInt64, false),
            LanceField::new("lineText", LanceDataType::Utf8, false),
            LanceField::new("score", LanceDataType::Float64, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceStringArray::from(crate_names)),
            Arc::new(LanceStringArray::from(project_names)),
            Arc::new(LanceStringArray::from(root_labels)),
            Arc::new(LanceStringArray::from(navigation_targets_json)),
            Arc::new(LanceUInt64Array::from(lines)),
            Arc::new(LanceUInt64Array::from(columns)),
            Arc::new(LanceStringArray::from(line_texts)),
            Arc::new(LanceFloat64Array::from(scores)),
        ],
    )
    .map_err(|error| error.to_string())
}
