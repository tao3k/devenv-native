//! Julia parser-summary Arrow contract facade and row identity types.

pub(crate) const JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN: &str = "request_id";
pub(crate) const JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN: &str = "source_id";
pub(crate) const JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN: &str = "source_text";

pub(crate) const JULIA_PARSER_SUMMARY_KIND_COLUMN: &str = "summary_kind";
pub(crate) const JULIA_PARSER_SUMMARY_BACKEND_COLUMN: &str = "backend";
pub(crate) const JULIA_PARSER_SUMMARY_SUCCESS_COLUMN: &str = "success";
pub(crate) const JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN: &str = "primary_name";
pub(crate) const JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN: &str = "error_message";
pub(crate) const JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN: &str = "module_name";
pub(crate) const JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN: &str = "module_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN: &str = "item_group";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN: &str = "item_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN: &str = "item_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN: &str = "item_signature";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN: &str = "item_target_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN: &str = "item_target_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN: &str = "item_target_path";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN: &str =
    "item_target_line_start";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN: &str = "item_target_line_end";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN: &str = "item_dependency_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN: &str = "item_dependency_form";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN: &str =
    "item_dependency_target";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN: &str =
    "item_dependency_is_relative";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN: &str =
    "item_dependency_relative_level";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN: &str =
    "item_dependency_local_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN: &str =
    "item_dependency_parent";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN: &str =
    "item_dependency_member";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN: &str = "item_dependency_alias";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN: &str = "item_content";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN: &str = "item_reexported";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN: &str = "item_path";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN: &str = "item_binding_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_MODULE_NAME_COLUMN: &str = "item_module_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_MODULE_PATH_COLUMN: &str = "item_module_path";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN: &str = "item_owner_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_OWNER_KIND_COLUMN: &str = "item_owner_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN: &str = "item_owner_path";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN: &str = "item_top_level";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN: &str = "item_line_start";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN: &str = "item_line_end";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TYPE_KIND_COLUMN: &str = "item_type_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TYPE_PARAMETERS_COLUMN: &str = "item_type_parameters";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_TYPE_SUPERTYPE_COLUMN: &str = "item_type_supertype";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PRIMITIVE_BITS_COLUMN: &str = "item_primitive_bits";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_KIND_COLUMN: &str = "item_parameter_kind";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_TYPE_NAME_COLUMN: &str =
    "item_parameter_type_name";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_DEFAULT_VALUE_COLUMN: &str =
    "item_parameter_default_value";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_TYPED_COLUMN: &str =
    "item_parameter_is_typed";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_DEFAULTED_COLUMN: &str =
    "item_parameter_is_defaulted";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_VARARG_COLUMN: &str =
    "item_parameter_is_vararg";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN: &str =
    "item_function_positional_arity";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN: &str =
    "item_function_keyword_arity";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_FUNCTION_HAS_VARARGS_COLUMN: &str =
    "item_function_has_varargs";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_FUNCTION_WHERE_PARAMS_COLUMN: &str =
    "item_function_where_params";
pub(crate) const JULIA_PARSER_SUMMARY_ITEM_FUNCTION_RETURN_TYPE_COLUMN: &str =
    "item_function_return_type";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JuliaParserSummaryRequestRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) source_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JuliaParserSummaryResponseRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) summary_kind: String,
    pub(crate) backend: String,
    pub(crate) success: bool,
    pub(crate) primary_name: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) module_name: Option<String>,
    pub(crate) module_kind: Option<String>,
    pub(crate) item_group: Option<String>,
    pub(crate) item_name: Option<String>,
    pub(crate) item_kind: Option<String>,
    pub(crate) item_signature: Option<String>,
    pub(crate) item_target_kind: Option<String>,
    pub(crate) item_target_name: Option<String>,
    pub(crate) item_target_path: Option<String>,
    pub(crate) item_target_line_start: Option<i64>,
    pub(crate) item_target_line_end: Option<i64>,
    pub(crate) item_dependency_kind: Option<String>,
    pub(crate) item_dependency_form: Option<String>,
    pub(crate) item_dependency_target: Option<String>,
    pub(crate) item_dependency_is_relative: Option<bool>,
    pub(crate) item_dependency_relative_level: Option<i32>,
    pub(crate) item_dependency_local_name: Option<String>,
    pub(crate) item_dependency_parent: Option<String>,
    pub(crate) item_dependency_member: Option<String>,
    pub(crate) item_dependency_alias: Option<String>,
    pub(crate) item_content: Option<String>,
    pub(crate) item_reexported: Option<bool>,
    pub(crate) item_path: Option<String>,
    pub(crate) item_binding_kind: Option<String>,
    pub(crate) item_module_name: Option<String>,
    pub(crate) item_module_path: Option<String>,
    pub(crate) item_owner_name: Option<String>,
    pub(crate) item_owner_kind: Option<String>,
    pub(crate) item_owner_path: Option<String>,
    pub(crate) item_top_level: Option<bool>,
    pub(crate) item_line_start: Option<i64>,
    pub(crate) item_line_end: Option<i64>,
    pub(crate) item_type_kind: Option<String>,
    pub(crate) item_type_parameters: Option<String>,
    pub(crate) item_type_supertype: Option<String>,
    pub(crate) item_primitive_bits: Option<i32>,
    pub(crate) item_parameter_kind: Option<String>,
    pub(crate) item_parameter_type_name: Option<String>,
    pub(crate) item_parameter_default_value: Option<String>,
    pub(crate) item_parameter_is_typed: Option<bool>,
    pub(crate) item_parameter_is_defaulted: Option<bool>,
    pub(crate) item_parameter_is_vararg: Option<bool>,
    pub(crate) item_function_positional_arity: Option<i32>,
    pub(crate) item_function_keyword_arity: Option<i32>,
    pub(crate) item_function_has_varargs: Option<bool>,
    pub(crate) item_function_where_params: Option<String>,
    pub(crate) item_function_return_type: Option<String>,
}
