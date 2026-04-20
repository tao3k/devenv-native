use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
    LanceUInt64Array,
};

use crate::gateway::studio::types::AstSearchHit;

pub(super) fn build_ast_hits_flight_batch(
    hits: &[AstSearchHit],
) -> Result<LanceRecordBatch, String> {
    let names = hits.iter().map(|hit| hit.name.as_str()).collect::<Vec<_>>();
    let signatures = hits
        .iter()
        .map(|hit| hit.signature.as_str())
        .collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.as_str()).collect::<Vec<_>>();
    let languages = hits
        .iter()
        .map(|hit| hit.language.as_str())
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
    let node_kinds = hits
        .iter()
        .map(|hit| hit.node_kind.as_deref())
        .collect::<Vec<_>>();
    let owner_titles = hits
        .iter()
        .map(|hit| hit.owner_title.as_deref())
        .collect::<Vec<_>>();
    let navigation_targets_json = hits
        .iter()
        .map(|hit| serde_json::to_string(&hit.navigation_target).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

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
            LanceField::new("lineStart", LanceDataType::UInt64, false),
            LanceField::new("lineEnd", LanceDataType::UInt64, false),
            LanceField::new("score", LanceDataType::Float64, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(signatures)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceStringArray::from(crate_names)),
            Arc::new(LanceStringArray::from(project_names)),
            Arc::new(LanceStringArray::from(root_labels)),
            Arc::new(LanceStringArray::from(node_kinds)),
            Arc::new(LanceStringArray::from(owner_titles)),
            Arc::new(LanceStringArray::from(
                navigation_targets_json
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceUInt64Array::from(
                hits.iter()
                    .map(|hit| hit.line_start as u64)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceUInt64Array::from(
                hits.iter()
                    .map(|hit| hit.line_end as u64)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceFloat64Array::from(
                hits.iter().map(|hit| hit.score).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| format!("failed to build AST-search Flight batch: {error}"))
}
