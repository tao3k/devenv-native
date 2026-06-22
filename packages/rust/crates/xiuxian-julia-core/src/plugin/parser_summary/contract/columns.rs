//! Julia parser-summary response column decoding.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::values::{
    optional_bool_values, optional_bool_values_or_missing, optional_int_values_or_missing,
    optional_int32_values, optional_int32_values_or_missing, optional_utf8_values,
    optional_utf8_values_or_missing, required_bool_values, required_utf8_values,
};
use super::{
    JULIA_PARSER_SUMMARY_BACKEND_COLUMN, JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN, JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_HAS_VARARGS_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_RETURN_TYPE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_WHERE_PARAMS_COLUMN, JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN, JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN, JULIA_PARSER_SUMMARY_ITEM_MODULE_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_MODULE_PATH_COLUMN, JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_OWNER_KIND_COLUMN, JULIA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_DEFAULT_VALUE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_DEFAULTED_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_TYPED_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_VARARG_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PARAMETER_TYPE_NAME_COLUMN, JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_PRIMITIVE_BITS_COLUMN, JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN, JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN, JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN, JULIA_PARSER_SUMMARY_ITEM_TYPE_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TYPE_PARAMETERS_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TYPE_SUPERTYPE_COLUMN, JULIA_PARSER_SUMMARY_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN, JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
    JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, JULIA_PARSER_SUMMARY_SUCCESS_COLUMN,
    JuliaParserSummaryResponseRow,
};

pub(super) struct JuliaParserSummaryResponseColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    error_message: Vec<Option<String>>,
    module_name: Vec<Option<String>>,
    module_kind: Vec<Option<String>>,
    item_group: Vec<Option<String>>,
    item_name: Vec<Option<String>>,
    item_kind: Vec<Option<String>>,
    item_signature: Vec<Option<String>>,
    item_target_kind: Vec<Option<String>>,
    item_target_name: Vec<Option<String>>,
    item_target_path: Vec<Option<String>>,
    item_target_line_start: Vec<Option<i64>>,
    item_target_line_end: Vec<Option<i64>>,
    item_dependency_kind: Vec<Option<String>>,
    item_dependency_form: Vec<Option<String>>,
    item_dependency_target: Vec<Option<String>>,
    item_dependency_is_relative: Vec<Option<bool>>,
    item_dependency_relative_level: Vec<Option<i32>>,
    item_dependency_local_name: Vec<Option<String>>,
    item_dependency_parent: Vec<Option<String>>,
    item_dependency_member: Vec<Option<String>>,
    item_dependency_alias: Vec<Option<String>>,
    item_content: Vec<Option<String>>,
    item_reexported: Vec<Option<bool>>,
    item_path: Vec<Option<String>>,
    item_binding_kind: Vec<Option<String>>,
    item_module_name: Vec<Option<String>>,
    item_module_path: Vec<Option<String>>,
    item_owner_name: Vec<Option<String>>,
    item_owner_kind: Vec<Option<String>>,
    item_owner_path: Vec<Option<String>>,
    item_top_level: Vec<Option<bool>>,
    item_line_start: Vec<Option<i64>>,
    item_line_end: Vec<Option<i64>>,
    item_type_kind: Vec<Option<String>>,
    item_type_parameters: Vec<Option<String>>,
    item_type_supertype: Vec<Option<String>>,
    item_primitive_bits: Vec<Option<i32>>,
    item_parameter_kind: Vec<Option<String>>,
    item_parameter_type_name: Vec<Option<String>>,
    item_parameter_default_value: Vec<Option<String>>,
    item_parameter_is_typed: Vec<Option<bool>>,
    item_parameter_is_defaulted: Vec<Option<bool>>,
    item_parameter_is_vararg: Vec<Option<bool>>,
    item_function_positional_arity: Vec<Option<i32>>,
    item_function_keyword_arity: Vec<Option<i32>>,
    item_function_has_varargs: Vec<Option<bool>>,
    item_function_where_params: Vec<Option<String>>,
    item_function_return_type: Vec<Option<String>>,
}

pub(super) struct JuliaParserSummaryBaseColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    error_message: Vec<Option<String>>,
    module_name: Vec<Option<String>>,
    module_kind: Vec<Option<String>>,
    item_group: Vec<Option<String>>,
    item_name: Vec<Option<String>>,
    item_kind: Vec<Option<String>>,
    item_signature: Vec<Option<String>>,
    item_content: Vec<Option<String>>,
    item_reexported: Vec<Option<bool>>,
    item_path: Vec<Option<String>>,
    item_binding_kind: Vec<Option<String>>,
    item_module_name: Vec<Option<String>>,
    item_module_path: Vec<Option<String>>,
    item_owner_name: Vec<Option<String>>,
    item_owner_kind: Vec<Option<String>>,
    item_owner_path: Vec<Option<String>>,
    item_top_level: Vec<Option<bool>>,
    item_line_start: Vec<Option<i64>>,
    item_line_end: Vec<Option<i64>>,
}

pub(super) struct JuliaParserSummaryHeaderColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    error_message: Vec<Option<String>>,
    module_name: Vec<Option<String>>,
    module_kind: Vec<Option<String>>,
}

pub(super) struct JuliaParserSummaryItemColumns {
    group: Vec<Option<String>>,
    name: Vec<Option<String>>,
    kind: Vec<Option<String>>,
    signature: Vec<Option<String>>,
    content: Vec<Option<String>>,
    reexported: Vec<Option<bool>>,
    path: Vec<Option<String>>,
    binding_kind: Vec<Option<String>>,
    module_name: Vec<Option<String>>,
    module_path: Vec<Option<String>>,
    owner_name: Vec<Option<String>>,
    owner_kind: Vec<Option<String>>,
    owner_path: Vec<Option<String>>,
    top_level: Vec<Option<bool>>,
    line_start: Vec<Option<i64>>,
    line_end: Vec<Option<i64>>,
}

pub(super) struct JuliaParserSummaryTargetColumns {
    kind: Vec<Option<String>>,
    name: Vec<Option<String>>,
    path: Vec<Option<String>>,
    line_start: Vec<Option<i64>>,
    line_end: Vec<Option<i64>>,
}

pub(super) struct JuliaParserSummaryDependencyColumns {
    kind: Vec<Option<String>>,
    form: Vec<Option<String>>,
    target: Vec<Option<String>>,
    is_relative: Vec<Option<bool>>,
    relative_level: Vec<Option<i32>>,
    local_name: Vec<Option<String>>,
    parent: Vec<Option<String>>,
    member: Vec<Option<String>>,
    alias: Vec<Option<String>>,
}

pub(super) struct JuliaParserSummaryTypeColumns {
    type_kind: Vec<Option<String>>,
    type_parameters: Vec<Option<String>>,
    type_supertype: Vec<Option<String>>,
    primitive_bits: Vec<Option<i32>>,
    param_kind: Vec<Option<String>>,
    param_type_name: Vec<Option<String>>,
    param_default_value: Vec<Option<String>>,
    param_is_typed: Vec<Option<bool>>,
    param_is_defaulted: Vec<Option<bool>>,
    param_is_vararg: Vec<Option<bool>>,
    positional_arity: Vec<Option<i32>>,
    keyword_arity: Vec<Option<i32>>,
    has_varargs: Vec<Option<bool>>,
    where_params: Vec<Option<String>>,
    return_type: Vec<Option<String>>,
}

impl JuliaParserSummaryBaseColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        let header = JuliaParserSummaryHeaderColumns::read(batch)?;
        let item = JuliaParserSummaryItemColumns::read(batch)?;
        Ok(Self {
            request_id: header.request_id,
            source_id: header.source_id,
            summary_kind: header.summary_kind,
            backend: header.backend,
            success: header.success,
            primary_name: header.primary_name,
            error_message: header.error_message,
            module_name: header.module_name,
            module_kind: header.module_kind,
            item_group: item.group,
            item_name: item.name,
            item_kind: item.kind,
            item_signature: item.signature,
            item_content: item.content,
            item_reexported: item.reexported,
            item_path: item.path,
            item_binding_kind: item.binding_kind,
            item_module_name: item.module_name,
            item_module_path: item.module_path,
            item_owner_name: item.owner_name,
            item_owner_kind: item.owner_kind,
            item_owner_path: item.owner_path,
            item_top_level: item.top_level,
            item_line_start: item.line_start,
            item_line_end: item.line_end,
        })
    }
}

impl JuliaParserSummaryHeaderColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            request_id: required_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
                "response",
            )?,
            source_id: required_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
                "response",
            )?,
            summary_kind: required_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_KIND_COLUMN,
                "response",
            )?,
            backend: required_utf8_values(batch, JULIA_PARSER_SUMMARY_BACKEND_COLUMN, "response")?,
            success: required_bool_values(batch, JULIA_PARSER_SUMMARY_SUCCESS_COLUMN, "response")?,
            primary_name: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN,
                "response",
            )?,
            error_message: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
                "response",
            )?,
            module_name: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN,
                "response",
            )?,
            module_kind: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN,
                "response",
            )?,
        })
    }
}

impl JuliaParserSummaryItemColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            group: optional_utf8_values(batch, JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN, "response")?,
            name: optional_utf8_values(batch, JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN, "response")?,
            kind: optional_utf8_values(batch, JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN, "response")?,
            signature: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN,
                "response",
            )?,
            content: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN,
                "response",
            )?,
            reexported: optional_bool_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN,
                "response",
            )?,
            path: optional_utf8_values(batch, JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN, "response")?,
            binding_kind: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN,
                "response",
            )?,
            module_name: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_MODULE_NAME_COLUMN,
                "response",
            )?,
            module_path: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_MODULE_PATH_COLUMN,
                "response",
            )?,
            owner_name: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN,
                "response",
            )?,
            owner_kind: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_OWNER_KIND_COLUMN,
                "response",
            )?,
            owner_path: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
                "response",
            )?,
            top_level: optional_bool_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
                "response",
            )?,
            line_start: optional_int_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
                "response",
            )?,
            line_end: optional_int_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
                "response",
            )?,
        })
    }
}

impl JuliaParserSummaryTargetColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            kind: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN,
                "response",
            )?,
            name: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN,
                "response",
            )?,
            path: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN,
                "response",
            )?,
            line_start: optional_int_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN,
                "response",
            )?,
            line_end: optional_int_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN,
                "response",
            )?,
        })
    }
}

impl JuliaParserSummaryDependencyColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            kind: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN,
                "response",
            )?,
            form: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
                "response",
            )?,
            target: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
                "response",
            )?,
            is_relative: optional_bool_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN,
                "response",
            )?,
            relative_level: optional_int32_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN,
                "response",
            )?,
            local_name: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
                "response",
            )?,
            parent: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN,
                "response",
            )?,
            member: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN,
                "response",
            )?,
            alias: optional_utf8_values(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
                "response",
            )?,
        })
    }
}

impl JuliaParserSummaryTypeColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            type_kind: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TYPE_KIND_COLUMN,
                "response",
            )?,
            type_parameters: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TYPE_PARAMETERS_COLUMN,
                "response",
            )?,
            type_supertype: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_TYPE_SUPERTYPE_COLUMN,
                "response",
            )?,
            primitive_bits: optional_int32_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PRIMITIVE_BITS_COLUMN,
                "response",
            )?,
            param_kind: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_KIND_COLUMN,
                "response",
            )?,
            param_type_name: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_TYPE_NAME_COLUMN,
                "response",
            )?,
            param_default_value: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_DEFAULT_VALUE_COLUMN,
                "response",
            )?,
            param_is_typed: optional_bool_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_TYPED_COLUMN,
                "response",
            )?,
            param_is_defaulted: optional_bool_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_DEFAULTED_COLUMN,
                "response",
            )?,
            param_is_vararg: optional_bool_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_PARAMETER_IS_VARARG_COLUMN,
                "response",
            )?,
            positional_arity: optional_int32_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN,
                "response",
            )?,
            keyword_arity: optional_int32_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN,
                "response",
            )?,
            has_varargs: optional_bool_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_FUNCTION_HAS_VARARGS_COLUMN,
                "response",
            )?,
            where_params: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_FUNCTION_WHERE_PARAMS_COLUMN,
                "response",
            )?,
            return_type: optional_utf8_values_or_missing(
                batch,
                JULIA_PARSER_SUMMARY_ITEM_FUNCTION_RETURN_TYPE_COLUMN,
                "response",
            )?,
        })
    }
}

impl JuliaParserSummaryResponseColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        let base = JuliaParserSummaryBaseColumns::read(batch)?;
        let target = JuliaParserSummaryTargetColumns::read(batch)?;
        let dependency = JuliaParserSummaryDependencyColumns::read(batch)?;
        let details = JuliaParserSummaryTypeColumns::read(batch)?;
        Ok(Self {
            request_id: base.request_id,
            source_id: base.source_id,
            summary_kind: base.summary_kind,
            backend: base.backend,
            success: base.success,
            primary_name: base.primary_name,
            error_message: base.error_message,
            module_name: base.module_name,
            module_kind: base.module_kind,
            item_group: base.item_group,
            item_name: base.item_name,
            item_kind: base.item_kind,
            item_signature: base.item_signature,
            item_target_kind: target.kind,
            item_target_name: target.name,
            item_target_path: target.path,
            item_target_line_start: target.line_start,
            item_target_line_end: target.line_end,
            item_dependency_kind: dependency.kind,
            item_dependency_form: dependency.form,
            item_dependency_target: dependency.target,
            item_dependency_is_relative: dependency.is_relative,
            item_dependency_relative_level: dependency.relative_level,
            item_dependency_local_name: dependency.local_name,
            item_dependency_parent: dependency.parent,
            item_dependency_member: dependency.member,
            item_dependency_alias: dependency.alias,
            item_content: base.item_content,
            item_reexported: base.item_reexported,
            item_path: base.item_path,
            item_binding_kind: base.item_binding_kind,
            item_module_name: base.item_module_name,
            item_module_path: base.item_module_path,
            item_owner_name: base.item_owner_name,
            item_owner_kind: base.item_owner_kind,
            item_owner_path: base.item_owner_path,
            item_top_level: base.item_top_level,
            item_line_start: base.item_line_start,
            item_line_end: base.item_line_end,
            item_type_kind: details.type_kind,
            item_type_parameters: details.type_parameters,
            item_type_supertype: details.type_supertype,
            item_primitive_bits: details.primitive_bits,
            item_parameter_kind: details.param_kind,
            item_parameter_type_name: details.param_type_name,
            item_parameter_default_value: details.param_default_value,
            item_parameter_is_typed: details.param_is_typed,
            item_parameter_is_defaulted: details.param_is_defaulted,
            item_parameter_is_vararg: details.param_is_vararg,
            item_function_positional_arity: details.positional_arity,
            item_function_keyword_arity: details.keyword_arity,
            item_function_has_varargs: details.has_varargs,
            item_function_where_params: details.where_params,
            item_function_return_type: details.return_type,
        })
    }

    pub(super) fn into_rows(self) -> Vec<JuliaParserSummaryResponseRow> {
        let row_count = self.request_id.len();
        (0..row_count)
            .map(|row_index| JuliaParserSummaryResponseRow {
                request_id: self.request_id[row_index].clone(),
                source_id: self.source_id[row_index].clone(),
                summary_kind: self.summary_kind[row_index].clone(),
                backend: self.backend[row_index].clone(),
                success: self.success[row_index],
                primary_name: self.primary_name[row_index].clone(),
                error_message: self.error_message[row_index].clone(),
                module_name: self.module_name[row_index].clone(),
                module_kind: self.module_kind[row_index].clone(),
                item_group: self.item_group[row_index].clone(),
                item_name: self.item_name[row_index].clone(),
                item_kind: self.item_kind[row_index].clone(),
                item_signature: self.item_signature[row_index].clone(),
                item_target_kind: self.item_target_kind[row_index].clone(),
                item_target_name: self.item_target_name[row_index].clone(),
                item_target_path: self.item_target_path[row_index].clone(),
                item_target_line_start: self.item_target_line_start[row_index],
                item_target_line_end: self.item_target_line_end[row_index],
                item_dependency_kind: self.item_dependency_kind[row_index].clone(),
                item_dependency_form: self.item_dependency_form[row_index].clone(),
                item_dependency_target: self.item_dependency_target[row_index].clone(),
                item_dependency_is_relative: self.item_dependency_is_relative[row_index],
                item_dependency_relative_level: self.item_dependency_relative_level[row_index],
                item_dependency_local_name: self.item_dependency_local_name[row_index].clone(),
                item_dependency_parent: self.item_dependency_parent[row_index].clone(),
                item_dependency_member: self.item_dependency_member[row_index].clone(),
                item_dependency_alias: self.item_dependency_alias[row_index].clone(),
                item_content: self.item_content[row_index].clone(),
                item_reexported: self.item_reexported[row_index],
                item_path: self.item_path[row_index].clone(),
                item_binding_kind: self.item_binding_kind[row_index].clone(),
                item_module_name: self.item_module_name[row_index].clone(),
                item_module_path: self.item_module_path[row_index].clone(),
                item_owner_name: self.item_owner_name[row_index].clone(),
                item_owner_kind: self.item_owner_kind[row_index].clone(),
                item_owner_path: self.item_owner_path[row_index].clone(),
                item_top_level: self.item_top_level[row_index],
                item_line_start: self.item_line_start[row_index],
                item_line_end: self.item_line_end[row_index],
                item_type_kind: self.item_type_kind[row_index].clone(),
                item_type_parameters: self.item_type_parameters[row_index].clone(),
                item_type_supertype: self.item_type_supertype[row_index].clone(),
                item_primitive_bits: self.item_primitive_bits[row_index],
                item_parameter_kind: self.item_parameter_kind[row_index].clone(),
                item_parameter_type_name: self.item_parameter_type_name[row_index].clone(),
                item_parameter_default_value: self.item_parameter_default_value[row_index].clone(),
                item_parameter_is_typed: self.item_parameter_is_typed[row_index],
                item_parameter_is_defaulted: self.item_parameter_is_defaulted[row_index],
                item_parameter_is_vararg: self.item_parameter_is_vararg[row_index],
                item_function_positional_arity: self.item_function_positional_arity[row_index],
                item_function_keyword_arity: self.item_function_keyword_arity[row_index],
                item_function_has_varargs: self.item_function_has_varargs[row_index],
                item_function_where_params: self.item_function_where_params[row_index].clone(),
                item_function_return_type: self.item_function_return_type[row_index].clone(),
            })
            .collect()
    }
}
