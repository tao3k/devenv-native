//! Modelica parser-summary Arrow contract facade and row identity types.

pub(crate) const MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN: &str = "request_id";
pub(crate) const MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN: &str = "source_id";
pub(crate) const MODELICA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN: &str = "source_text";
pub(crate) const MODELICA_PARSER_SUMMARY_KIND_COLUMN: &str = "summary_kind";
pub(crate) const MODELICA_PARSER_SUMMARY_BACKEND_COLUMN: &str = "backend";
pub(crate) const MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN: &str = "success";
pub(crate) const MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN: &str = "primary_name";
pub(crate) const MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN: &str = "error_message";
pub(crate) const MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN: &str = "class_name";
pub(crate) const MODELICA_PARSER_SUMMARY_RESTRICTION_COLUMN: &str = "restriction";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_GROUP_COLUMN: &str = "item_group";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_NAME_COLUMN: &str = "item_name";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_KIND_COLUMN: &str = "item_kind";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN: &str = "item_signature";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN: &str = "item_dependency_form";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN: &str =
    "item_dependency_target";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN: &str =
    "item_dependency_alias";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN: &str =
    "item_dependency_local_name";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_TEXT_COLUMN: &str = "item_text";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN: &str = "item_line_start";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN: &str = "item_line_end";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN: &str = "item_owner_name";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN: &str = "item_owner_path";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_VISIBILITY_COLUMN: &str = "item_visibility";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_TYPE_NAME_COLUMN: &str = "item_type_name";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_VARIABILITY_COLUMN: &str = "item_variability";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DIRECTION_COLUMN: &str = "item_direction";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_COMPONENT_KIND_COLUMN: &str = "item_component_kind";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_ARRAY_DIMENSIONS_COLUMN: &str =
    "item_array_dimensions";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_DEFAULT_VALUE_COLUMN: &str = "item_default_value";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_START_VALUE_COLUMN: &str = "item_start_value";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_MODIFIER_NAMES_COLUMN: &str = "item_modifier_names";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_UNIT_COLUMN: &str = "item_unit";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_CLASS_PATH_COLUMN: &str = "item_class_path";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN: &str = "item_top_level";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_IS_PARTIAL_COLUMN: &str = "item_is_partial";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_IS_FINAL_COLUMN: &str = "item_is_final";
pub(crate) const MODELICA_PARSER_SUMMARY_ITEM_IS_ENCAPSULATED_COLUMN: &str = "item_is_encapsulated";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaParserSummaryRequestRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) source_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaParserSummaryResponseRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) summary_kind: String,
    pub(crate) backend: String,
    pub(crate) success: bool,
    pub(crate) primary_name: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) class_name: Option<String>,
    pub(crate) restriction: Option<String>,
    pub(crate) item_group: Option<String>,
    pub(crate) item_name: Option<String>,
    pub(crate) item_kind: Option<String>,
    pub(crate) item_signature: Option<String>,
    pub(crate) item_dependency_form: Option<String>,
    pub(crate) item_dependency_target: Option<String>,
    pub(crate) item_dependency_alias: Option<String>,
    pub(crate) item_dependency_local_name: Option<String>,
    pub(crate) item_text: Option<String>,
    pub(crate) item_line_start: Option<i64>,
    pub(crate) item_line_end: Option<i64>,
    pub(crate) item_owner_name: Option<String>,
    pub(crate) item_owner_path: Option<String>,
    pub(crate) item_visibility: Option<String>,
    pub(crate) item_type_name: Option<String>,
    pub(crate) item_variability: Option<String>,
    pub(crate) item_direction: Option<String>,
    pub(crate) item_component_kind: Option<String>,
    pub(crate) item_array_dimensions: Option<String>,
    pub(crate) item_default_value: Option<String>,
    pub(crate) item_start_value: Option<String>,
    pub(crate) item_modifier_names: Option<String>,
    pub(crate) item_unit: Option<String>,
    pub(crate) item_class_path: Option<String>,
    pub(crate) item_top_level: Option<bool>,
    pub(crate) item_is_partial: Option<bool>,
    pub(crate) item_is_final: Option<bool>,
    pub(crate) item_is_encapsulated: Option<bool>,
}
