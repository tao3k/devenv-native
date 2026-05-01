use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::rows::ParserSummaryRequest;

const SUMMARY_KIND: &str = "modelica_file_summary";

#[derive(Debug, Clone)]
struct ModelicaRow {
    request_id: String,
    source_id: String,
    backend: String,
    success: bool,
    class_name: Option<String>,
    restriction: Option<String>,
    item_group: Option<String>,
    item_name: Option<String>,
    item_kind: Option<String>,
    item_signature: Option<String>,
    item_text: Option<String>,
    item_line_start: Option<i64>,
    item_line_end: Option<i64>,
    item_owner_name: Option<String>,
    item_owner_path: Option<String>,
    item_visibility: Option<String>,
    item_class_path: Option<String>,
    item_top_level: Option<bool>,
    item_is_partial: Option<bool>,
    item_is_final: Option<bool>,
    item_is_encapsulated: Option<bool>,
}

impl ModelicaRow {
    fn base(request: &ParserSummaryRequest, class: Option<&ParsedModelicaClass>) -> Self {
        Self {
            request_id: request.request_id.clone(),
            source_id: request.source_id.clone(),
            backend: "rust-test-fixture".to_string(),
            success: true,
            class_name: class.map(|class| class.name.clone()),
            restriction: class.map(|class| class.restriction.clone()),
            item_group: None,
            item_name: None,
            item_kind: None,
            item_signature: None,
            item_text: None,
            item_line_start: None,
            item_line_end: None,
            item_owner_name: None,
            item_owner_path: None,
            item_visibility: None,
            item_class_path: None,
            item_top_level: None,
            item_is_partial: None,
            item_is_final: None,
            item_is_encapsulated: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedModelicaClass {
    name: String,
    restriction: String,
    line_start: i64,
    line_end: i64,
}

pub(crate) fn response_batch_for_requests(
    requests: &[ParserSummaryRequest],
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let rows = requests.iter().flat_map(response_rows).collect::<Vec<_>>();
    rows_to_batch(rows.as_slice())
}

fn response_rows(request: &ParserSummaryRequest) -> Vec<ModelicaRow> {
    let parsed_class = parse_modelica_class(request.source_text.as_str());
    let Some(class) = parsed_class.as_ref() else {
        return vec![ModelicaRow::base(request, None)];
    };
    let mut row = ModelicaRow::base(request, Some(class));
    row.item_group = Some("symbol".to_string());
    row.item_name = Some(class.name.clone());
    row.item_kind = Some(class.restriction.clone());
    row.item_signature = Some(format!("{} {}", class.restriction, class.name));
    row.item_line_start = Some(class.line_start);
    row.item_line_end = Some(class.line_end);
    row.item_visibility = Some("public".to_string());
    row.item_class_path = Some(class.name.clone());
    row.item_top_level = Some(true);
    row.item_is_partial = Some(false);
    row.item_is_final = Some(false);
    row.item_is_encapsulated = Some(false);
    vec![row]
}

fn parse_modelica_class(source: &str) -> Option<ParsedModelicaClass> {
    let lines = source.lines().collect::<Vec<_>>();
    let (index, restriction, name) = lines.iter().enumerate().find_map(|(index, line)| {
        parse_class_declaration(line.trim()).map(|(restriction, name)| (index, restriction, name))
    })?;
    let line_start = i64::try_from(index + 1).unwrap_or(i64::MAX);
    let line_end = find_modelica_end(&lines, index, name.as_str()).unwrap_or(line_start);
    Some(ParsedModelicaClass {
        name,
        restriction,
        line_start,
        line_end,
    })
}

fn parse_class_declaration(line: &str) -> Option<(String, String)> {
    ["model", "record", "block", "connector", "type", "package"]
        .into_iter()
        .find_map(|restriction| {
            line.strip_prefix(restriction)
                .and_then(|rest| rest.split_whitespace().next())
                .filter(|name| !name.is_empty())
                .map(|name| {
                    (
                        restriction.to_string(),
                        name.trim_end_matches(';').to_string(),
                    )
                })
        })
}

fn find_modelica_end(lines: &[&str], start_index: usize, class_name: &str) -> Option<i64> {
    let expected = format!("end {class_name}");
    lines
        .iter()
        .enumerate()
        .skip(start_index + 1)
        .find(|(_, line)| line.trim().trim_end_matches(';') == expected)
        .and_then(|(index, _)| i64::try_from(index + 1).ok())
}

fn rows_to_batch(rows: &[ModelicaRow]) -> Result<RecordBatch, arrow::error::ArrowError> {
    RecordBatch::try_new(
        response_schema(),
        vec![
            required_utf8(rows, |row| &row.request_id),
            required_utf8(rows, |row| &row.source_id),
            repeated_utf8(rows, SUMMARY_KIND),
            required_utf8(rows, |row| &row.backend),
            required_bool(rows, |row| row.success),
            optional_utf8(rows, |row| row.class_name.as_deref()),
            null_utf8(rows),
            optional_utf8(rows, |row| row.class_name.as_deref()),
            optional_utf8(rows, |row| row.restriction.as_deref()),
            optional_utf8(rows, |row| row.item_group.as_deref()),
            optional_utf8(rows, |row| row.item_name.as_deref()),
            optional_utf8(rows, |row| row.item_kind.as_deref()),
            optional_utf8(rows, |row| row.item_signature.as_deref()),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            optional_utf8(rows, |row| row.item_text.as_deref()),
            optional_i64(rows, |row| row.item_line_start),
            optional_i64(rows, |row| row.item_line_end),
            optional_utf8(rows, |row| row.item_owner_name.as_deref()),
            optional_utf8(rows, |row| row.item_owner_path.as_deref()),
            optional_utf8(rows, |row| row.item_visibility.as_deref()),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            null_utf8(rows),
            optional_utf8(rows, |row| row.item_class_path.as_deref()),
            optional_bool(rows, |row| row.item_top_level),
            optional_bool(rows, |row| row.item_is_partial),
            optional_bool(rows, |row| row.item_is_final),
            optional_bool(rows, |row| row.item_is_encapsulated),
        ],
    )
}

fn response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        utf8("request_id", false),
        utf8("source_id", false),
        utf8("summary_kind", false),
        utf8("backend", false),
        bool_field("success", false),
        utf8("primary_name", true),
        utf8("error_message", true),
        utf8("class_name", true),
        utf8("restriction", true),
        utf8("item_group", true),
        utf8("item_name", true),
        utf8("item_kind", true),
        utf8("item_signature", true),
        utf8("item_dependency_form", true),
        utf8("item_dependency_target", true),
        utf8("item_dependency_alias", true),
        utf8("item_dependency_local_name", true),
        utf8("item_text", true),
        int64("item_line_start"),
        int64("item_line_end"),
        utf8("item_owner_name", true),
        utf8("item_owner_path", true),
        utf8("item_visibility", true),
        utf8("item_type_name", true),
        utf8("item_variability", true),
        utf8("item_direction", true),
        utf8("item_component_kind", true),
        utf8("item_array_dimensions", true),
        utf8("item_default_value", true),
        utf8("item_start_value", true),
        utf8("item_modifier_names", true),
        utf8("item_unit", true),
        utf8("item_class_path", true),
        bool_field("item_top_level", true),
        bool_field("item_is_partial", true),
        bool_field("item_is_final", true),
        bool_field("item_is_encapsulated", true),
    ]))
}

fn utf8(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

fn bool_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Boolean, nullable)
}

fn int64(name: &str) -> Field {
    Field::new(name, DataType::Int64, true)
}

fn required_utf8(rows: &[ModelicaRow], value: impl Fn(&ModelicaRow) -> &str) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn repeated_utf8(rows: &[ModelicaRow], value: &str) -> ArrayRef {
    Arc::new(StringArray::from(vec![value; rows.len()]))
}

fn optional_utf8<'a>(
    rows: &'a [ModelicaRow],
    value: impl Fn(&'a ModelicaRow) -> Option<&'a str>,
) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn null_utf8(rows: &[ModelicaRow]) -> ArrayRef {
    Arc::new(StringArray::from(vec![None::<&str>; rows.len()]))
}

fn required_bool(rows: &[ModelicaRow], value: impl Fn(&ModelicaRow) -> bool) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn optional_bool(rows: &[ModelicaRow], value: impl Fn(&ModelicaRow) -> Option<bool>) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn optional_i64(rows: &[ModelicaRow], value: impl Fn(&ModelicaRow) -> Option<i64>) -> ArrayRef {
    Arc::new(Int64Array::from(rows.iter().map(value).collect::<Vec<_>>()))
}
