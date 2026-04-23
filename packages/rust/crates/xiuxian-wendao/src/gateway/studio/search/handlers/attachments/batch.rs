use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
};

use crate::gateway::studio::types::AttachmentSearchHit;

pub(super) fn build_attachment_hits_flight_batch(
    hits: &[AttachmentSearchHit],
) -> Result<LanceRecordBatch, String> {
    let names = hits.iter().map(|hit| hit.name.clone()).collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>();
    let source_ids = hits
        .iter()
        .map(|hit| hit.source_id.clone())
        .collect::<Vec<_>>();
    let source_stems = hits
        .iter()
        .map(|hit| hit.source_stem.clone())
        .collect::<Vec<_>>();
    let source_titles = hits
        .iter()
        .map(|hit| hit.source_title.clone())
        .collect::<Vec<_>>();
    let navigation_targets_json = hits
        .iter()
        .map(|hit| serde_json::to_string(&hit.navigation_target).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let source_paths = hits
        .iter()
        .map(|hit| hit.source_path.clone())
        .collect::<Vec<_>>();
    let attachment_ids = hits
        .iter()
        .map(|hit| hit.attachment_id.clone())
        .collect::<Vec<_>>();
    let attachment_paths = hits
        .iter()
        .map(|hit| hit.attachment_path.clone())
        .collect::<Vec<_>>();
    let attachment_names = hits
        .iter()
        .map(|hit| hit.attachment_name.clone())
        .collect::<Vec<_>>();
    let attachment_exts = hits
        .iter()
        .map(|hit| hit.attachment_ext.clone())
        .collect::<Vec<_>>();
    let kinds = hits.iter().map(|hit| hit.kind.clone()).collect::<Vec<_>>();
    let scores = hits.iter().map(|hit| hit.score).collect::<Vec<_>>();
    let vision_snippets = hits
        .iter()
        .map(|hit| hit.vision_snippet.as_deref())
        .collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("name", LanceDataType::Utf8, false),
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("sourceId", LanceDataType::Utf8, false),
            LanceField::new("sourceStem", LanceDataType::Utf8, false),
            LanceField::new("sourceTitle", LanceDataType::Utf8, false),
            LanceField::new("navigationTargetJson", LanceDataType::Utf8, true),
            LanceField::new("sourcePath", LanceDataType::Utf8, false),
            LanceField::new("attachmentId", LanceDataType::Utf8, false),
            LanceField::new("attachmentPath", LanceDataType::Utf8, false),
            LanceField::new("attachmentName", LanceDataType::Utf8, false),
            LanceField::new("attachmentExt", LanceDataType::Utf8, false),
            LanceField::new("kind", LanceDataType::Utf8, false),
            LanceField::new("score", LanceDataType::Float64, false),
            LanceField::new("visionSnippet", LanceDataType::Utf8, true),
        ])),
        vec![
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceStringArray::from(source_ids)),
            Arc::new(LanceStringArray::from(source_stems)),
            Arc::new(LanceStringArray::from(source_titles)),
            Arc::new(LanceStringArray::from(
                navigation_targets_json
                    .iter()
                    .map(|value| Some(value.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceStringArray::from(source_paths)),
            Arc::new(LanceStringArray::from(attachment_ids)),
            Arc::new(LanceStringArray::from(attachment_paths)),
            Arc::new(LanceStringArray::from(attachment_names)),
            Arc::new(LanceStringArray::from(attachment_exts)),
            Arc::new(LanceStringArray::from(kinds)),
            Arc::new(LanceFloat64Array::from(scores)),
            Arc::new(LanceStringArray::from(vision_snippets)),
        ],
    )
    .map_err(|error| error.to_string())
}
