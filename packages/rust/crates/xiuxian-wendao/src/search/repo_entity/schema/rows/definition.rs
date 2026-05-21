//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::sync::Arc;

use xiuxian_db_store::{LanceDataType, LanceField, LanceSchema};

use crate::search::repo_entity::schema::{
    COLUMN_ATTRIBUTES_JSON, COLUMN_AUDIT_STATUS, COLUMN_ENTITY_KIND, COLUMN_HIERARCHICAL_URI,
    COLUMN_HIERARCHY, COLUMN_HIT_JSON, COLUMN_ID, COLUMN_IMPLICIT_BACKLINK_ITEMS_JSON,
    COLUMN_IMPLICIT_BACKLINKS, COLUMN_LANGUAGE, COLUMN_LINE_END, COLUMN_LINE_START,
    COLUMN_MODULE_ID, COLUMN_NAME, COLUMN_NAME_FOLDED, COLUMN_PATH, COLUMN_PATH_FOLDED,
    COLUMN_PROJECTION_PAGE_IDS, COLUMN_QUALIFIED_NAME, COLUMN_QUALIFIED_NAME_FOLDED,
    COLUMN_RELATED_MODULES_FOLDED, COLUMN_RELATED_SYMBOLS_FOLDED, COLUMN_SALIENCY_SCORE,
    COLUMN_SEARCH_TEXT, COLUMN_SIGNATURE, COLUMN_SIGNATURE_FOLDED, COLUMN_SUMMARY,
    COLUMN_SUMMARY_FOLDED, COLUMN_SYMBOL_KIND, COLUMN_VERIFICATION_STATE,
};

/// Builds the Lance schema for repo-entity rows.
pub(crate) fn repo_entity_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ENTITY_KIND, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_QUALIFIED_NAME, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_QUALIFIED_NAME_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LANGUAGE, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SYMBOL_KIND, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_MODULE_ID, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_SIGNATURE, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_SIGNATURE_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SUMMARY, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_SUMMARY_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_RELATED_SYMBOLS_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_RELATED_MODULES_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LINE_START, LanceDataType::UInt32, true),
        LanceField::new(COLUMN_LINE_END, LanceDataType::UInt32, true),
        LanceField::new(COLUMN_AUDIT_STATUS, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_VERIFICATION_STATE, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_ATTRIBUTES_JSON, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_HIERARCHICAL_URI, LanceDataType::Utf8, true),
        LanceField::new(
            COLUMN_HIERARCHY,
            LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
            false,
        ),
        LanceField::new(
            COLUMN_IMPLICIT_BACKLINKS,
            LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
            false,
        ),
        LanceField::new(
            COLUMN_IMPLICIT_BACKLINK_ITEMS_JSON,
            LanceDataType::Utf8,
            true,
        ),
        LanceField::new(
            COLUMN_PROJECTION_PAGE_IDS,
            LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
            false,
        ),
        LanceField::new(COLUMN_SALIENCY_SCORE, LanceDataType::Float64, false),
        LanceField::new(COLUMN_SEARCH_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_HIT_JSON, LanceDataType::Utf8, false),
    ]))
}
