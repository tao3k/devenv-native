//! Modelica parser-summary response column decoding.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::values::{
    optional_bool_values, optional_int_values, optional_utf8_values, required_bool_values,
    required_utf8_values,
};
use super::{
    MODELICA_PARSER_SUMMARY_BACKEND_COLUMN, MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_ARRAY_DIMENSIONS_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_CLASS_PATH_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_COMPONENT_KIND_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEFAULT_VALUE_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DIRECTION_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_GROUP_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_IS_ENCAPSULATED_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_IS_FINAL_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_IS_PARTIAL_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_KIND_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_MODIFIER_NAMES_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_START_VALUE_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_TEXT_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_TYPE_NAME_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_UNIT_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_VARIABILITY_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_VISIBILITY_COLUMN, MODELICA_PARSER_SUMMARY_KIND_COLUMN,
    MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
    MODELICA_PARSER_SUMMARY_RESTRICTION_COLUMN, MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
    MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN, ModelicaParserSummaryResponseRow,
};

pub(super) struct ModelicaParserSummaryResponseColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    error_message: Vec<Option<String>>,
    class_name: Vec<Option<String>>,
    restriction: Vec<Option<String>>,
    item_group: Vec<Option<String>>,
    item_name: Vec<Option<String>>,
    item_kind: Vec<Option<String>>,
    item_signature: Vec<Option<String>>,
    item_dependency_form: Vec<Option<String>>,
    item_dependency_target: Vec<Option<String>>,
    item_dependency_alias: Vec<Option<String>>,
    item_dependency_local_name: Vec<Option<String>>,
    item_text: Vec<Option<String>>,
    item_line_start: Vec<Option<i64>>,
    item_line_end: Vec<Option<i64>>,
    item_owner_name: Vec<Option<String>>,
    item_owner_path: Vec<Option<String>>,
    item_visibility: Vec<Option<String>>,
    item_type_name: Vec<Option<String>>,
    item_variability: Vec<Option<String>>,
    item_direction: Vec<Option<String>>,
    item_component_kind: Vec<Option<String>>,
    item_array_dimensions: Vec<Option<String>>,
    item_default_value: Vec<Option<String>>,
    item_start_value: Vec<Option<String>>,
    item_modifier_names: Vec<Option<String>>,
    item_unit: Vec<Option<String>>,
    item_class_path: Vec<Option<String>>,
    item_top_level: Vec<Option<bool>>,
    item_is_partial: Vec<Option<bool>>,
    item_is_final: Vec<Option<bool>>,
    item_is_encapsulated: Vec<Option<bool>>,
}

pub(super) struct ModelicaParserSummaryBaseColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    error_message: Vec<Option<String>>,
    class_name: Vec<Option<String>>,
    restriction: Vec<Option<String>>,
    item_group: Vec<Option<String>>,
    item_name: Vec<Option<String>>,
    item_kind: Vec<Option<String>>,
    item_signature: Vec<Option<String>>,
    item_text: Vec<Option<String>>,
    item_line_start: Vec<Option<i64>>,
    item_line_end: Vec<Option<i64>>,
    item_owner_name: Vec<Option<String>>,
    item_owner_path: Vec<Option<String>>,
}

pub(super) struct ModelicaParserSummaryDependencyColumns {
    form: Vec<Option<String>>,
    target: Vec<Option<String>>,
    alias: Vec<Option<String>>,
    local_name: Vec<Option<String>>,
}

pub(super) struct ModelicaParserSummaryDetailColumns {
    visibility: Vec<Option<String>>,
    type_name: Vec<Option<String>>,
    variability: Vec<Option<String>>,
    direction: Vec<Option<String>>,
    component_kind: Vec<Option<String>>,
    array_dimensions: Vec<Option<String>>,
    default_value: Vec<Option<String>>,
    start_value: Vec<Option<String>>,
    modifier_names: Vec<Option<String>>,
    unit: Vec<Option<String>>,
    class_path: Vec<Option<String>>,
    top_level: Vec<Option<bool>>,
    is_partial: Vec<Option<bool>>,
    is_final: Vec<Option<bool>>,
    is_encapsulated: Vec<Option<bool>>,
}

impl ModelicaParserSummaryBaseColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            request_id: required_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
                "response",
            )?,
            source_id: required_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
                "response",
            )?,
            summary_kind: required_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_KIND_COLUMN,
                "response",
            )?,
            backend: required_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_BACKEND_COLUMN,
                "response",
            )?,
            success: required_bool_values(
                batch,
                MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN,
                "response",
            )?,
            primary_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN,
                "response",
            )?,
            error_message: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
                "response",
            )?,
            class_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN,
                "response",
            )?,
            restriction: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_RESTRICTION_COLUMN,
                "response",
            )?,
            item_group: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_GROUP_COLUMN,
                "response",
            )?,
            item_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_NAME_COLUMN,
                "response",
            )?,
            item_kind: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_KIND_COLUMN,
                "response",
            )?,
            item_signature: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN,
                "response",
            )?,
            item_text: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_TEXT_COLUMN,
                "response",
            )?,
            item_line_start: optional_int_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
                "response",
            )?,
            item_line_end: optional_int_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
                "response",
            )?,
            item_owner_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN,
                "response",
            )?,
            item_owner_path: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
                "response",
            )?,
        })
    }
}

impl ModelicaParserSummaryDependencyColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            form: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
                "response",
            )?,
            target: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
                "response",
            )?,
            alias: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
                "response",
            )?,
            local_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
                "response",
            )?,
        })
    }
}

impl ModelicaParserSummaryDetailColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            visibility: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_VISIBILITY_COLUMN,
                "response",
            )?,
            type_name: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_TYPE_NAME_COLUMN,
                "response",
            )?,
            variability: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_VARIABILITY_COLUMN,
                "response",
            )?,
            direction: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DIRECTION_COLUMN,
                "response",
            )?,
            component_kind: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_COMPONENT_KIND_COLUMN,
                "response",
            )?,
            array_dimensions: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_ARRAY_DIMENSIONS_COLUMN,
                "response",
            )?,
            default_value: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_DEFAULT_VALUE_COLUMN,
                "response",
            )?,
            start_value: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_START_VALUE_COLUMN,
                "response",
            )?,
            modifier_names: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_MODIFIER_NAMES_COLUMN,
                "response",
            )?,
            unit: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_UNIT_COLUMN,
                "response",
            )?,
            class_path: optional_utf8_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_CLASS_PATH_COLUMN,
                "response",
            )?,
            top_level: optional_bool_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
                "response",
            )?,
            is_partial: optional_bool_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_IS_PARTIAL_COLUMN,
                "response",
            )?,
            is_final: optional_bool_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_IS_FINAL_COLUMN,
                "response",
            )?,
            is_encapsulated: optional_bool_values(
                batch,
                MODELICA_PARSER_SUMMARY_ITEM_IS_ENCAPSULATED_COLUMN,
                "response",
            )?,
        })
    }
}

impl ModelicaParserSummaryResponseColumns {
    pub(super) fn read(batch: &RecordBatch) -> Result<Self, RepoIntelligenceError> {
        let base = ModelicaParserSummaryBaseColumns::read(batch)?;
        let dependency = ModelicaParserSummaryDependencyColumns::read(batch)?;
        let details = ModelicaParserSummaryDetailColumns::read(batch)?;
        Ok(Self {
            request_id: base.request_id,
            source_id: base.source_id,
            summary_kind: base.summary_kind,
            backend: base.backend,
            success: base.success,
            primary_name: base.primary_name,
            error_message: base.error_message,
            class_name: base.class_name,
            restriction: base.restriction,
            item_group: base.item_group,
            item_name: base.item_name,
            item_kind: base.item_kind,
            item_signature: base.item_signature,
            item_dependency_form: dependency.form,
            item_dependency_target: dependency.target,
            item_dependency_alias: dependency.alias,
            item_dependency_local_name: dependency.local_name,
            item_text: base.item_text,
            item_line_start: base.item_line_start,
            item_line_end: base.item_line_end,
            item_owner_name: base.item_owner_name,
            item_owner_path: base.item_owner_path,
            item_visibility: details.visibility,
            item_type_name: details.type_name,
            item_variability: details.variability,
            item_direction: details.direction,
            item_component_kind: details.component_kind,
            item_array_dimensions: details.array_dimensions,
            item_default_value: details.default_value,
            item_start_value: details.start_value,
            item_modifier_names: details.modifier_names,
            item_unit: details.unit,
            item_class_path: details.class_path,
            item_top_level: details.top_level,
            item_is_partial: details.is_partial,
            item_is_final: details.is_final,
            item_is_encapsulated: details.is_encapsulated,
        })
    }

    pub(super) fn into_rows(self) -> Vec<ModelicaParserSummaryResponseRow> {
        let row_count = self.request_id.len();
        (0..row_count)
            .map(|row_index| ModelicaParserSummaryResponseRow {
                request_id: self.request_id[row_index].clone(),
                source_id: self.source_id[row_index].clone(),
                summary_kind: self.summary_kind[row_index].clone(),
                backend: self.backend[row_index].clone(),
                success: self.success[row_index],
                primary_name: self.primary_name[row_index].clone(),
                error_message: self.error_message[row_index].clone(),
                class_name: self.class_name[row_index].clone(),
                restriction: self.restriction[row_index].clone(),
                item_group: self.item_group[row_index].clone(),
                item_name: self.item_name[row_index].clone(),
                item_kind: self.item_kind[row_index].clone(),
                item_signature: self.item_signature[row_index].clone(),
                item_dependency_form: self.item_dependency_form[row_index].clone(),
                item_dependency_target: self.item_dependency_target[row_index].clone(),
                item_dependency_alias: self.item_dependency_alias[row_index].clone(),
                item_dependency_local_name: self.item_dependency_local_name[row_index].clone(),
                item_text: self.item_text[row_index].clone(),
                item_line_start: self.item_line_start[row_index],
                item_line_end: self.item_line_end[row_index],
                item_owner_name: self.item_owner_name[row_index].clone(),
                item_owner_path: self.item_owner_path[row_index].clone(),
                item_visibility: self.item_visibility[row_index].clone(),
                item_type_name: self.item_type_name[row_index].clone(),
                item_variability: self.item_variability[row_index].clone(),
                item_direction: self.item_direction[row_index].clone(),
                item_component_kind: self.item_component_kind[row_index].clone(),
                item_array_dimensions: self.item_array_dimensions[row_index].clone(),
                item_default_value: self.item_default_value[row_index].clone(),
                item_start_value: self.item_start_value[row_index].clone(),
                item_modifier_names: self.item_modifier_names[row_index].clone(),
                item_unit: self.item_unit[row_index].clone(),
                item_class_path: self.item_class_path[row_index].clone(),
                item_top_level: self.item_top_level[row_index],
                item_is_partial: self.item_is_partial[row_index],
                item_is_final: self.item_is_final[row_index],
                item_is_encapsulated: self.item_is_encapsulated[row_index],
            })
            .collect()
    }
}
