use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int32Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::schema;

#[derive(Debug, Clone)]
pub(crate) struct ParserSummaryRequest {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) source_text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ParserSummaryRow {
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

impl ParserSummaryRow {
    pub(crate) fn base(
        request: &ParserSummaryRequest,
        summary_kind: &str,
        module_name: Option<&str>,
    ) -> Self {
        let module_name = module_name.map(str::to_string);
        Self {
            request_id: request.request_id.clone(),
            source_id: request.source_id.clone(),
            summary_kind: summary_kind.to_string(),
            backend: "rust-test-fixture".to_string(),
            success: true,
            primary_name: module_name.clone(),
            error_message: None,
            module_kind: module_name.as_ref().map(|_| "module".to_string()),
            module_name,
            item_group: None,
            item_name: None,
            item_kind: None,
            item_signature: None,
            item_content: None,
            item_reexported: None,
            item_path: None,
            item_binding_kind: None,
            item_module_name: None,
            item_module_path: None,
            item_owner_name: None,
            item_owner_kind: None,
            item_owner_path: None,
            item_top_level: None,
            item_line_start: None,
            item_line_end: None,
            item_target_kind: None,
            item_target_name: None,
            item_target_path: None,
            item_target_line_start: None,
            item_target_line_end: None,
            item_dependency_kind: None,
            item_dependency_form: None,
            item_dependency_target: None,
            item_dependency_is_relative: None,
            item_dependency_relative_level: None,
            item_dependency_local_name: None,
            item_dependency_parent: None,
            item_dependency_member: None,
            item_dependency_alias: None,
            item_type_kind: None,
            item_type_parameters: None,
            item_type_supertype: None,
            item_primitive_bits: None,
            item_parameter_kind: None,
            item_parameter_type_name: None,
            item_parameter_default_value: None,
            item_parameter_is_typed: None,
            item_parameter_is_defaulted: None,
            item_parameter_is_vararg: None,
            item_function_positional_arity: None,
            item_function_keyword_arity: None,
            item_function_has_varargs: None,
            item_function_where_params: None,
            item_function_return_type: None,
        }
    }
}

pub(crate) fn rows_to_batch(
    rows: &[ParserSummaryRow],
) -> Result<RecordBatch, arrow::error::ArrowError> {
    RecordBatch::try_new(
        schema::response_schema(),
        vec![
            required_utf8(rows, |row| &row.request_id),
            required_utf8(rows, |row| &row.source_id),
            required_utf8(rows, |row| &row.summary_kind),
            required_utf8(rows, |row| &row.backend),
            required_bool(rows, |row| row.success),
            optional_utf8(rows, |row| row.primary_name.as_deref()),
            optional_utf8(rows, |row| row.error_message.as_deref()),
            optional_utf8(rows, |row| row.module_name.as_deref()),
            optional_utf8(rows, |row| row.module_kind.as_deref()),
            optional_utf8(rows, |row| row.item_group.as_deref()),
            optional_utf8(rows, |row| row.item_name.as_deref()),
            optional_utf8(rows, |row| row.item_kind.as_deref()),
            optional_utf8(rows, |row| row.item_signature.as_deref()),
            optional_utf8(rows, |row| row.item_content.as_deref()),
            optional_bool(rows, |row| row.item_reexported),
            optional_utf8(rows, |row| row.item_path.as_deref()),
            optional_utf8(rows, |row| row.item_binding_kind.as_deref()),
            optional_utf8(rows, |row| row.item_module_name.as_deref()),
            optional_utf8(rows, |row| row.item_module_path.as_deref()),
            optional_utf8(rows, |row| row.item_owner_name.as_deref()),
            optional_utf8(rows, |row| row.item_owner_kind.as_deref()),
            optional_utf8(rows, |row| row.item_owner_path.as_deref()),
            optional_bool(rows, |row| row.item_top_level),
            optional_i64(rows, |row| row.item_line_start),
            optional_i64(rows, |row| row.item_line_end),
            optional_utf8(rows, |row| row.item_target_kind.as_deref()),
            optional_utf8(rows, |row| row.item_target_name.as_deref()),
            optional_utf8(rows, |row| row.item_target_path.as_deref()),
            optional_i64(rows, |row| row.item_target_line_start),
            optional_i64(rows, |row| row.item_target_line_end),
            optional_utf8(rows, |row| row.item_dependency_kind.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_form.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_target.as_deref()),
            optional_bool(rows, |row| row.item_dependency_is_relative),
            optional_i32(rows, |row| row.item_dependency_relative_level),
            optional_utf8(rows, |row| row.item_dependency_local_name.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_parent.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_member.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_alias.as_deref()),
            optional_utf8(rows, |row| row.item_type_kind.as_deref()),
            optional_utf8(rows, |row| row.item_type_parameters.as_deref()),
            optional_utf8(rows, |row| row.item_type_supertype.as_deref()),
            optional_i32(rows, |row| row.item_primitive_bits),
            optional_utf8(rows, |row| row.item_parameter_kind.as_deref()),
            optional_utf8(rows, |row| row.item_parameter_type_name.as_deref()),
            optional_utf8(rows, |row| row.item_parameter_default_value.as_deref()),
            optional_bool(rows, |row| row.item_parameter_is_typed),
            optional_bool(rows, |row| row.item_parameter_is_defaulted),
            optional_bool(rows, |row| row.item_parameter_is_vararg),
            optional_i32(rows, |row| row.item_function_positional_arity),
            optional_i32(rows, |row| row.item_function_keyword_arity),
            optional_bool(rows, |row| row.item_function_has_varargs),
            optional_utf8(rows, |row| row.item_function_where_params.as_deref()),
            optional_utf8(rows, |row| row.item_function_return_type.as_deref()),
        ],
    )
}

fn required_utf8(rows: &[ParserSummaryRow], value: impl Fn(&ParserSummaryRow) -> &str) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn optional_utf8<'a>(
    rows: &'a [ParserSummaryRow],
    value: impl Fn(&'a ParserSummaryRow) -> Option<&'a str>,
) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn required_bool(rows: &[ParserSummaryRow], value: impl Fn(&ParserSummaryRow) -> bool) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn optional_bool(
    rows: &[ParserSummaryRow],
    value: impl Fn(&ParserSummaryRow) -> Option<bool>,
) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn optional_i32(
    rows: &[ParserSummaryRow],
    value: impl Fn(&ParserSummaryRow) -> Option<i32>,
) -> ArrayRef {
    Arc::new(Int32Array::from(rows.iter().map(value).collect::<Vec<_>>()))
}

fn optional_i64(
    rows: &[ParserSummaryRow],
    value: impl Fn(&ParserSummaryRow) -> Option<i64>,
) -> ArrayRef {
    Arc::new(Int64Array::from(rows.iter().map(value).collect::<Vec<_>>()))
}
