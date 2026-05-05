use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};

pub(crate) const REQUEST_ID: &str = "request_id";
pub(crate) const SOURCE_ID: &str = "source_id";
pub(crate) const SOURCE_TEXT: &str = "source_text";
pub(crate) const SUMMARY_KIND: &str = "summary_kind";
pub(crate) const BACKEND: &str = "backend";
pub(crate) const SUCCESS: &str = "success";
pub(crate) const PRIMARY_NAME: &str = "primary_name";
pub(crate) const ERROR_MESSAGE: &str = "error_message";
pub(crate) const MODULE_NAME: &str = "module_name";
pub(crate) const MODULE_KIND: &str = "module_kind";
pub(crate) const ITEM_GROUP: &str = "item_group";
pub(crate) const ITEM_NAME: &str = "item_name";
pub(crate) const ITEM_KIND: &str = "item_kind";
pub(crate) const ITEM_SIGNATURE: &str = "item_signature";
pub(crate) const ITEM_TARGET_KIND: &str = "item_target_kind";
pub(crate) const ITEM_TARGET_NAME: &str = "item_target_name";
pub(crate) const ITEM_TARGET_PATH: &str = "item_target_path";
pub(crate) const ITEM_TARGET_LINE_START: &str = "item_target_line_start";
pub(crate) const ITEM_TARGET_LINE_END: &str = "item_target_line_end";
pub(crate) const ITEM_DEPENDENCY_KIND: &str = "item_dependency_kind";
pub(crate) const ITEM_DEPENDENCY_FORM: &str = "item_dependency_form";
pub(crate) const ITEM_DEPENDENCY_TARGET: &str = "item_dependency_target";
pub(crate) const ITEM_DEPENDENCY_IS_RELATIVE: &str = "item_dependency_is_relative";
pub(crate) const ITEM_DEPENDENCY_RELATIVE_LEVEL: &str = "item_dependency_relative_level";
pub(crate) const ITEM_DEPENDENCY_LOCAL_NAME: &str = "item_dependency_local_name";
pub(crate) const ITEM_DEPENDENCY_PARENT: &str = "item_dependency_parent";
pub(crate) const ITEM_DEPENDENCY_MEMBER: &str = "item_dependency_member";
pub(crate) const ITEM_DEPENDENCY_ALIAS: &str = "item_dependency_alias";
pub(crate) const ITEM_CONTENT: &str = "item_content";
pub(crate) const ITEM_REEXPORTED: &str = "item_reexported";
pub(crate) const ITEM_PATH: &str = "item_path";
pub(crate) const ITEM_BINDING_KIND: &str = "item_binding_kind";
pub(crate) const ITEM_MODULE_NAME: &str = "item_module_name";
pub(crate) const ITEM_MODULE_PATH: &str = "item_module_path";
pub(crate) const ITEM_OWNER_NAME: &str = "item_owner_name";
pub(crate) const ITEM_OWNER_KIND: &str = "item_owner_kind";
pub(crate) const ITEM_OWNER_PATH: &str = "item_owner_path";
pub(crate) const ITEM_TOP_LEVEL: &str = "item_top_level";
pub(crate) const ITEM_LINE_START: &str = "item_line_start";
pub(crate) const ITEM_LINE_END: &str = "item_line_end";
pub(crate) const ITEM_TYPE_KIND: &str = "item_type_kind";
pub(crate) const ITEM_TYPE_PARAMETERS: &str = "item_type_parameters";
pub(crate) const ITEM_TYPE_SUPERTYPE: &str = "item_type_supertype";
pub(crate) const ITEM_PRIMITIVE_BITS: &str = "item_primitive_bits";
pub(crate) const ITEM_PARAMETER_KIND: &str = "item_parameter_kind";
pub(crate) const ITEM_PARAMETER_TYPE_NAME: &str = "item_parameter_type_name";
pub(crate) const ITEM_PARAMETER_DEFAULT_VALUE: &str = "item_parameter_default_value";
pub(crate) const ITEM_PARAMETER_IS_TYPED: &str = "item_parameter_is_typed";
pub(crate) const ITEM_PARAMETER_IS_DEFAULTED: &str = "item_parameter_is_defaulted";
pub(crate) const ITEM_PARAMETER_IS_VARARG: &str = "item_parameter_is_vararg";
pub(crate) const ITEM_FUNCTION_POSITIONAL_ARITY: &str = "item_function_positional_arity";
pub(crate) const ITEM_FUNCTION_KEYWORD_ARITY: &str = "item_function_keyword_arity";
pub(crate) const ITEM_FUNCTION_HAS_VARARGS: &str = "item_function_has_varargs";
pub(crate) const ITEM_FUNCTION_WHERE_PARAMS: &str = "item_function_where_params";
pub(crate) const ITEM_FUNCTION_RETURN_TYPE: &str = "item_function_return_type";

pub(crate) fn response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        utf8(REQUEST_ID, false),
        utf8(SOURCE_ID, false),
        utf8(SUMMARY_KIND, false),
        utf8(BACKEND, false),
        bool_field(SUCCESS, false),
        utf8(PRIMARY_NAME, true),
        utf8(ERROR_MESSAGE, true),
        utf8(MODULE_NAME, true),
        utf8(MODULE_KIND, true),
        utf8(ITEM_GROUP, true),
        utf8(ITEM_NAME, true),
        utf8(ITEM_KIND, true),
        utf8(ITEM_SIGNATURE, true),
        utf8(ITEM_CONTENT, true),
        bool_field(ITEM_REEXPORTED, true),
        utf8(ITEM_PATH, true),
        utf8(ITEM_BINDING_KIND, true),
        utf8(ITEM_MODULE_NAME, true),
        utf8(ITEM_MODULE_PATH, true),
        utf8(ITEM_OWNER_NAME, true),
        utf8(ITEM_OWNER_KIND, true),
        utf8(ITEM_OWNER_PATH, true),
        bool_field(ITEM_TOP_LEVEL, true),
        int64(ITEM_LINE_START),
        int64(ITEM_LINE_END),
        utf8(ITEM_TARGET_KIND, true),
        utf8(ITEM_TARGET_NAME, true),
        utf8(ITEM_TARGET_PATH, true),
        int64(ITEM_TARGET_LINE_START),
        int64(ITEM_TARGET_LINE_END),
        utf8(ITEM_DEPENDENCY_KIND, true),
        utf8(ITEM_DEPENDENCY_FORM, true),
        utf8(ITEM_DEPENDENCY_TARGET, true),
        bool_field(ITEM_DEPENDENCY_IS_RELATIVE, true),
        int32(ITEM_DEPENDENCY_RELATIVE_LEVEL),
        utf8(ITEM_DEPENDENCY_LOCAL_NAME, true),
        utf8(ITEM_DEPENDENCY_PARENT, true),
        utf8(ITEM_DEPENDENCY_MEMBER, true),
        utf8(ITEM_DEPENDENCY_ALIAS, true),
        utf8(ITEM_TYPE_KIND, true),
        utf8(ITEM_TYPE_PARAMETERS, true),
        utf8(ITEM_TYPE_SUPERTYPE, true),
        int32(ITEM_PRIMITIVE_BITS),
        utf8(ITEM_PARAMETER_KIND, true),
        utf8(ITEM_PARAMETER_TYPE_NAME, true),
        utf8(ITEM_PARAMETER_DEFAULT_VALUE, true),
        bool_field(ITEM_PARAMETER_IS_TYPED, true),
        bool_field(ITEM_PARAMETER_IS_DEFAULTED, true),
        bool_field(ITEM_PARAMETER_IS_VARARG, true),
        int32(ITEM_FUNCTION_POSITIONAL_ARITY),
        int32(ITEM_FUNCTION_KEYWORD_ARITY),
        bool_field(ITEM_FUNCTION_HAS_VARARGS, true),
        utf8(ITEM_FUNCTION_WHERE_PARAMS, true),
        utf8(ITEM_FUNCTION_RETURN_TYPE, true),
    ]))
}

fn utf8(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

fn bool_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Boolean, nullable)
}

fn int32(name: &str) -> Field {
    Field::new(name, DataType::Int32, true)
}

fn int64(name: &str) -> Field {
    Field::new(name, DataType::Int64, true)
}
