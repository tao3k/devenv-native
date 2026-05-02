//! Modelica AST-query Arrow contract facade and row identity types.

const MODELICA_AST_QUERY_REQUEST_ID_COLUMN: &str = "request_id";
const MODELICA_AST_QUERY_SOURCE_ID_COLUMN: &str = "source_id";
const MODELICA_AST_QUERY_SOURCE_TEXT_COLUMN: &str = "source_text";
const MODELICA_AST_QUERY_NODE_KIND_COLUMN: &str = "node_kind";
const MODELICA_AST_QUERY_NAME_EQUALS_COLUMN: &str = "name_equals";
const MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN: &str = "name_contains";
const MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN: &str = "text_contains";
const MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN: &str = "signature_contains";
const MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN: &str = "attribute_key";
const MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN: &str = "attribute_equals";
const MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN: &str = "attribute_contains";
const MODELICA_AST_QUERY_LIMIT_COLUMN: &str = "limit";

const MODELICA_AST_QUERY_SUMMARY_KIND_COLUMN: &str = "summary_kind";
const MODELICA_AST_QUERY_BACKEND_COLUMN: &str = "backend";
const MODELICA_AST_QUERY_SUCCESS_COLUMN: &str = "success";
const MODELICA_AST_QUERY_PRIMARY_NAME_COLUMN: &str = "primary_name";
const MODELICA_AST_QUERY_MATCH_COUNT_COLUMN: &str = "match_count";
const MODELICA_AST_QUERY_ERROR_MESSAGE_COLUMN: &str = "error_message";
const MODELICA_AST_QUERY_MATCH_INDEX_COLUMN: &str = "match_index";
const MODELICA_AST_QUERY_MATCH_NODE_KIND_COLUMN: &str = "match_node_kind";
const MODELICA_AST_QUERY_MATCH_NAME_COLUMN: &str = "match_name";
const MODELICA_AST_QUERY_MATCH_TEXT_COLUMN: &str = "match_text";
const MODELICA_AST_QUERY_MATCH_SIGNATURE_COLUMN: &str = "match_signature";
const MODELICA_AST_QUERY_MATCH_PATH_COLUMN: &str = "match_path";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_KIND_COLUMN: &str = "match_dependency_kind";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_FORM_COLUMN: &str = "match_dependency_form";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_TARGET_COLUMN: &str = "match_dependency_target";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_LOCAL_NAME_COLUMN: &str = "match_dependency_local_name";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_PARENT_COLUMN: &str = "match_dependency_parent";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_MEMBER_COLUMN: &str = "match_dependency_member";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_ALIAS_COLUMN: &str = "match_dependency_alias";
const MODELICA_AST_QUERY_MATCH_LINE_START_COLUMN: &str = "match_line_start";
const MODELICA_AST_QUERY_MATCH_LINE_END_COLUMN: &str = "match_line_end";
const MODELICA_AST_QUERY_MATCH_OWNER_NAME_COLUMN: &str = "match_owner_name";
const MODELICA_AST_QUERY_MATCH_OWNER_PATH_COLUMN: &str = "match_owner_path";
const MODELICA_AST_QUERY_MATCH_CLASS_PATH_COLUMN: &str = "match_class_path";
const MODELICA_AST_QUERY_MATCH_TOP_LEVEL_COLUMN: &str = "match_top_level";
const MODELICA_AST_QUERY_MATCH_VISIBILITY_COLUMN: &str = "match_visibility";
const MODELICA_AST_QUERY_MATCH_TYPE_NAME_COLUMN: &str = "match_type_name";
const MODELICA_AST_QUERY_MATCH_VARIABILITY_COLUMN: &str = "match_variability";
const MODELICA_AST_QUERY_MATCH_DIRECTION_COLUMN: &str = "match_direction";
const MODELICA_AST_QUERY_MATCH_COMPONENT_KIND_COLUMN: &str = "match_component_kind";
const MODELICA_AST_QUERY_MATCH_ARRAY_DIMENSIONS_COLUMN: &str = "match_array_dimensions";
const MODELICA_AST_QUERY_MATCH_DEFAULT_VALUE_COLUMN: &str = "match_default_value";
const MODELICA_AST_QUERY_MATCH_START_VALUE_COLUMN: &str = "match_start_value";
const MODELICA_AST_QUERY_MATCH_MODIFIER_NAMES_COLUMN: &str = "match_modifier_names";
const MODELICA_AST_QUERY_MATCH_UNIT_COLUMN: &str = "match_unit";
const MODELICA_AST_QUERY_MATCH_IS_PARTIAL_COLUMN: &str = "match_is_partial";
const MODELICA_AST_QUERY_MATCH_IS_FINAL_COLUMN: &str = "match_is_final";
const MODELICA_AST_QUERY_MATCH_IS_ENCAPSULATED_COLUMN: &str = "match_is_encapsulated";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaAstQueryRequest {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) source_text: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaAstQueryResponseRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) summary_kind: String,
    pub(crate) backend: String,
    pub(crate) success: bool,
    pub(crate) primary_name: Option<String>,
    pub(crate) match_count: Option<i64>,
    pub(crate) error_message: Option<String>,
    pub(crate) match_index: Option<i64>,
    pub(crate) match_node_kind: Option<String>,
    pub(crate) match_name: Option<String>,
    pub(crate) match_text: Option<String>,
    pub(crate) match_signature: Option<String>,
    pub(crate) match_path: Option<String>,
    pub(crate) match_dependency_kind: Option<String>,
    pub(crate) match_dependency_form: Option<String>,
    pub(crate) match_dependency_target: Option<String>,
    pub(crate) match_dependency_local_name: Option<String>,
    pub(crate) match_dependency_parent: Option<String>,
    pub(crate) match_dependency_member: Option<String>,
    pub(crate) match_dependency_alias: Option<String>,
    pub(crate) match_line_start: Option<i64>,
    pub(crate) match_line_end: Option<i64>,
    pub(crate) match_owner_name: Option<String>,
    pub(crate) match_owner_path: Option<String>,
    pub(crate) match_class_path: Option<String>,
    pub(crate) match_top_level: Option<bool>,
    pub(crate) match_visibility: Option<String>,
    pub(crate) match_type_name: Option<String>,
    pub(crate) match_variability: Option<String>,
    pub(crate) match_direction: Option<String>,
    pub(crate) match_component_kind: Option<String>,
    pub(crate) match_array_dimensions: Option<String>,
    pub(crate) match_default_value: Option<String>,
    pub(crate) match_start_value: Option<String>,
    pub(crate) match_modifier_names: Option<String>,
    pub(crate) match_unit: Option<String>,
    pub(crate) match_is_partial: Option<bool>,
    pub(crate) match_is_final: Option<bool>,
    pub(crate) match_is_encapsulated: Option<bool>,
}

mod analysis;
mod batch;
mod rows;
mod values;

pub(crate) use analysis::decode_modelica_ast_query_analysis;
pub(crate) use batch::build_modelica_ast_query_request_batch;
pub(crate) use rows::decode_modelica_ast_query_response_rows;

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/ast_query_contract.rs"]
mod tests;
