use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::rows::ParserSummaryRequest;

const SUMMARY_KIND: &str = "modelica_file_summary";
const AST_QUERY_SUMMARY_KIND: &str = "modelica_ast_query";
const AST_QUERY_BACKEND: &str = "OMParser.jl";

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
    item_dependency_form: Option<String>,
    item_dependency_target: Option<String>,
    item_dependency_alias: Option<String>,
    item_dependency_local_name: Option<String>,
    item_text: Option<String>,
    item_line_start: Option<i64>,
    item_line_end: Option<i64>,
    item_owner_name: Option<String>,
    item_owner_path: Option<String>,
    item_visibility: Option<String>,
    item_type_name: Option<String>,
    item_variability: Option<String>,
    item_direction: Option<String>,
    item_component_kind: Option<String>,
    item_array_dimensions: Option<String>,
    item_default_value: Option<String>,
    item_start_value: Option<String>,
    item_modifier_names: Option<String>,
    item_unit: Option<String>,
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
            item_dependency_form: None,
            item_dependency_target: None,
            item_dependency_alias: None,
            item_dependency_local_name: None,
            item_text: None,
            item_line_start: None,
            item_line_end: None,
            item_owner_name: None,
            item_owner_path: None,
            item_visibility: None,
            item_type_name: None,
            item_variability: None,
            item_direction: None,
            item_component_kind: None,
            item_array_dimensions: None,
            item_default_value: None,
            item_start_value: None,
            item_modifier_names: None,
            item_unit: None,
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

#[derive(Debug, Clone)]
struct ParsedModelicaComponent {
    name: String,
    type_name: String,
    signature: String,
    variability: Option<String>,
    direction: Option<String>,
    component_kind: String,
    default_value: Option<String>,
    line_number: i64,
}

#[derive(Debug, Clone)]
struct ParsedModelicaImport {
    name: String,
    target: String,
    form: String,
    alias: Option<String>,
    local_name: Option<String>,
    line_number: i64,
}

pub(crate) fn response_batch_for_requests(
    requests: &[ParserSummaryRequest],
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let rows = requests.iter().flat_map(response_rows).collect::<Vec<_>>();
    rows_to_batch(rows.as_slice())
}

pub(crate) fn ast_query_response_batch_for_requests(
    requests: &[ParserSummaryRequest],
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let rows = requests.iter().flat_map(ast_query_rows).collect::<Vec<_>>();
    ast_query_rows_to_batch(rows.as_slice())
}

fn response_rows(request: &ParserSummaryRequest) -> Vec<ModelicaRow> {
    let parsed_class = parse_modelica_class(request.source_text.as_str());
    let Some(class) = parsed_class.as_ref() else {
        return vec![ModelicaRow::base(request, None)];
    };
    let mut rows = vec![class_row(request, class)];
    rows.extend(
        parse_modelica_classes(request.source_text.as_str())
            .into_iter()
            .filter(|candidate| candidate.line_start != class.line_start)
            .map(|candidate| class_row(request, &candidate)),
    );
    rows.extend(
        parse_modelica_imports(request.source_text.as_str())
            .into_iter()
            .map(|modelica_import| import_row(request, class, &modelica_import)),
    );
    rows.extend(
        parse_modelica_components(request.source_text.as_str())
            .into_iter()
            .map(|component| component_row(request, class, &component)),
    );
    rows
}

fn class_row(request: &ParserSummaryRequest, class: &ParsedModelicaClass) -> ModelicaRow {
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
    row
}

fn import_row(
    request: &ParserSummaryRequest,
    class: &ParsedModelicaClass,
    modelica_import: &ParsedModelicaImport,
) -> ModelicaRow {
    let mut row = ModelicaRow::base(request, Some(class));
    row.item_group = Some("import".to_string());
    row.item_name = Some(modelica_import.name.clone());
    row.item_dependency_form = Some(modelica_import.form.clone());
    row.item_dependency_target = Some(modelica_import.target.clone());
    row.item_dependency_alias.clone_from(&modelica_import.alias);
    row.item_dependency_local_name
        .clone_from(&modelica_import.local_name);
    row.item_line_start = Some(modelica_import.line_number);
    row.item_line_end = Some(modelica_import.line_number);
    row.item_owner_name = Some(class.name.clone());
    row.item_owner_path = Some(class.name.clone());
    row
}

fn component_row(
    request: &ParserSummaryRequest,
    class: &ParsedModelicaClass,
    component: &ParsedModelicaComponent,
) -> ModelicaRow {
    let mut row = ModelicaRow::base(request, Some(class));
    row.item_group = Some("symbol".to_string());
    row.item_name = Some(component.name.clone());
    row.item_kind = Some(component_kind_for_summary(component));
    row.item_signature = Some(component.signature.clone());
    row.item_line_start = Some(component.line_number);
    row.item_line_end = Some(component.line_number);
    row.item_owner_name = Some(class.name.clone());
    row.item_owner_path = Some(class.name.clone());
    row.item_visibility = Some("public".to_string());
    row.item_type_name = Some(component.type_name.clone());
    row.item_variability.clone_from(&component.variability);
    row.item_direction.clone_from(&component.direction);
    row.item_component_kind = Some(component.component_kind.clone());
    row.item_default_value.clone_from(&component.default_value);
    row.item_class_path = Some(class.name.clone());
    row.item_top_level = Some(false);
    row.item_is_partial = Some(false);
    row.item_is_final = Some(false);
    row.item_is_encapsulated = Some(false);
    row
}

fn component_kind_for_summary(component: &ParsedModelicaComponent) -> String {
    component
        .variability
        .as_deref()
        .filter(|value| matches!(*value, "constant" | "parameter"))
        .unwrap_or("parameter")
        .to_string()
}

fn parse_modelica_class(source: &str) -> Option<ParsedModelicaClass> {
    parse_modelica_classes(source).into_iter().next()
}

fn parse_modelica_classes(source: &str) -> Vec<ParsedModelicaClass> {
    let lines = source.lines().collect::<Vec<_>>();
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let (restriction, name) = parse_class_declaration(line.trim())?;
            let line_start = line_number(index);
            let line_end = find_modelica_end(&lines, index, name.as_str()).unwrap_or(line_start);
            Some(ParsedModelicaClass {
                name,
                restriction,
                line_start,
                line_end,
            })
        })
        .collect()
}

fn parse_modelica_imports(source: &str) -> Vec<ParsedModelicaImport> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| parse_modelica_import(line.trim(), line_number(index)))
        .collect()
}

fn parse_modelica_import(line: &str, line_number: i64) -> Option<ParsedModelicaImport> {
    let import = line.strip_prefix("import ")?.trim_end_matches(';').trim();
    let (alias, target) = import.split_once('=').map_or_else(
        || (None, import.to_string()),
        |(alias, target)| (Some(alias.trim().to_string()), target.trim().to_string()),
    );
    let (target, form, local_name) = if let Some(parent) = target.strip_suffix(".*") {
        let parent = parent.trim_end_matches('.');
        (
            parent.to_string(),
            "unqualified_import".to_string(),
            parent
                .rsplit('.')
                .next()
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        )
    } else {
        let local_name = target
            .rsplit('.')
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let form = if alias.is_some() {
            "named_import"
        } else {
            "qualified_import"
        };
        (target, form.to_string(), local_name)
    };
    let name = alias
        .clone()
        .or_else(|| local_name.clone())
        .unwrap_or_else(|| target.clone());
    Some(ParsedModelicaImport {
        name,
        target,
        form,
        alias,
        local_name,
        line_number,
    })
}

fn parse_modelica_components(source: &str) -> Vec<ParsedModelicaComponent> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| parse_modelica_component(line.trim(), line_number(index)))
        .collect()
}

fn parse_modelica_component(line: &str, line_number: i64) -> Option<ParsedModelicaComponent> {
    if !line.ends_with(';')
        || line.starts_with("within ")
        || line.starts_with("import ")
        || line.starts_with("end ")
        || parse_class_declaration(line).is_some()
    {
        return None;
    }
    let signature = line.to_string();
    let declaration = line
        .split_once("//")
        .map_or(line, |(declaration, _)| declaration)
        .trim_end_matches(';')
        .trim();
    let mut tokens = declaration.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 {
        return None;
    }
    let variability = take_optional_prefix(&mut tokens, &["constant", "parameter"]);
    let direction = take_optional_prefix(&mut tokens, &["input", "output"]);
    if tokens.len() < 2 {
        return None;
    }
    let type_name = tokens[0].to_string();
    let name = tokens[1]
        .split(['[', '(', '='])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();
    let default_value = declaration
        .split_once('=')
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let component_kind = variability
        .clone()
        .unwrap_or_else(|| "component".to_string());
    Some(ParsedModelicaComponent {
        name,
        type_name,
        signature,
        variability,
        direction,
        component_kind,
        default_value,
        line_number,
    })
}

fn take_optional_prefix(tokens: &mut Vec<&str>, candidates: &[&str]) -> Option<String> {
    if tokens
        .first()
        .is_some_and(|token| candidates.contains(token))
    {
        return Some(tokens.remove(0).to_string());
    }
    None
}

fn line_number(index: usize) -> i64 {
    i64::try_from(index + 1).unwrap_or(i64::MAX)
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

#[derive(Debug, Clone)]
struct AstQueryRow {
    request_id: String,
    source_id: String,
    primary_name: Option<String>,
    match_count: Option<i64>,
    match_index: Option<i64>,
    match_node_kind: Option<String>,
    match_name: Option<String>,
    match_text: Option<String>,
    match_signature: Option<String>,
    match_path: Option<String>,
    match_dependency_kind: Option<String>,
    match_dependency_form: Option<String>,
    match_dependency_target: Option<String>,
    match_dependency_local_name: Option<String>,
    match_dependency_parent: Option<String>,
    match_dependency_member: Option<String>,
    match_dependency_alias: Option<String>,
    match_line_start: Option<i64>,
    match_line_end: Option<i64>,
    match_owner_name: Option<String>,
    match_owner_path: Option<String>,
    match_class_path: Option<String>,
    match_top_level: Option<bool>,
    match_visibility: Option<String>,
    match_type_name: Option<String>,
    match_variability: Option<String>,
    match_direction: Option<String>,
    match_component_kind: Option<String>,
    match_array_dimensions: Option<String>,
    match_default_value: Option<String>,
    match_start_value: Option<String>,
    match_modifier_names: Option<String>,
    match_unit: Option<String>,
    match_is_partial: Option<bool>,
    match_is_final: Option<bool>,
    match_is_encapsulated: Option<bool>,
}

impl AstQueryRow {
    fn base(request: &ParserSummaryRequest, class: Option<&ParsedModelicaClass>) -> Self {
        Self {
            request_id: request.request_id.clone(),
            source_id: request.source_id.clone(),
            primary_name: class.map(|class| class.name.clone()),
            match_count: None,
            match_index: None,
            match_node_kind: None,
            match_name: None,
            match_text: None,
            match_signature: None,
            match_path: None,
            match_dependency_kind: None,
            match_dependency_form: None,
            match_dependency_target: None,
            match_dependency_local_name: None,
            match_dependency_parent: None,
            match_dependency_member: None,
            match_dependency_alias: None,
            match_line_start: None,
            match_line_end: None,
            match_owner_name: None,
            match_owner_path: None,
            match_class_path: None,
            match_top_level: None,
            match_visibility: None,
            match_type_name: None,
            match_variability: None,
            match_direction: None,
            match_component_kind: None,
            match_array_dimensions: None,
            match_default_value: None,
            match_start_value: None,
            match_modifier_names: None,
            match_unit: None,
            match_is_partial: None,
            match_is_final: None,
            match_is_encapsulated: None,
        }
    }
}

fn ast_query_rows(request: &ParserSummaryRequest) -> Vec<AstQueryRow> {
    let parsed_class = parse_modelica_class(request.source_text.as_str());
    let Some(class) = parsed_class.as_ref() else {
        return vec![AstQueryRow::base(request, None)];
    };
    let mut rows = vec![ast_query_class_row(request, class)];
    rows.extend(
        parse_modelica_classes(request.source_text.as_str())
            .into_iter()
            .filter(|candidate| candidate.line_start != class.line_start)
            .map(|candidate| ast_query_class_row(request, &candidate)),
    );
    rows.extend(
        parse_modelica_imports(request.source_text.as_str())
            .into_iter()
            .map(|modelica_import| ast_query_import_row(request, class, &modelica_import)),
    );
    rows.extend(
        parse_modelica_components(request.source_text.as_str())
            .into_iter()
            .map(|component| ast_query_component_row(request, class, &component)),
    );
    let match_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    for (index, row) in rows.iter_mut().enumerate() {
        row.match_count = Some(match_count);
        row.match_index = Some(i64::try_from(index).unwrap_or(i64::MAX));
    }
    rows
}

fn ast_query_class_row(request: &ParserSummaryRequest, class: &ParsedModelicaClass) -> AstQueryRow {
    let mut row = AstQueryRow::base(request, Some(class));
    row.match_node_kind = Some(class.restriction.clone());
    row.match_name = Some(class.name.clone());
    row.match_text = Some(format!("{} {}", class.restriction, class.name));
    row.match_signature = Some(format!("{} {}", class.restriction, class.name));
    row.match_path = Some(class.name.clone());
    row.match_line_start = Some(class.line_start);
    row.match_line_end = Some(class.line_end);
    row.match_owner_name = Some(class.name.clone());
    row.match_owner_path = Some(class.name.clone());
    row.match_class_path = Some(class.name.clone());
    row.match_top_level = Some(true);
    row.match_visibility = Some("public".to_string());
    row.match_is_partial = Some(false);
    row.match_is_final = Some(false);
    row.match_is_encapsulated = Some(false);
    row
}

fn ast_query_import_row(
    request: &ParserSummaryRequest,
    class: &ParsedModelicaClass,
    modelica_import: &ParsedModelicaImport,
) -> AstQueryRow {
    let mut row = AstQueryRow::base(request, Some(class));
    row.match_node_kind = Some("import".to_string());
    row.match_name = Some(modelica_import.name.clone());
    row.match_text = Some(format!("import {};", modelica_import.target));
    row.match_signature = row.match_text.clone();
    row.match_path = Some(class.name.clone());
    row.match_dependency_kind = Some("import".to_string());
    row.match_dependency_form = Some(modelica_import.form.clone());
    row.match_dependency_target = Some(modelica_import.target.clone());
    row.match_dependency_local_name
        .clone_from(&modelica_import.local_name);
    row.match_dependency_alias
        .clone_from(&modelica_import.alias);
    row.match_line_start = Some(modelica_import.line_number);
    row.match_line_end = Some(modelica_import.line_number);
    row.match_owner_name = Some(class.name.clone());
    row.match_owner_path = Some(class.name.clone());
    row.match_class_path = Some(class.name.clone());
    row
}

fn ast_query_component_row(
    request: &ParserSummaryRequest,
    class: &ParsedModelicaClass,
    component: &ParsedModelicaComponent,
) -> AstQueryRow {
    let mut row = AstQueryRow::base(request, Some(class));
    row.match_node_kind = Some("component".to_string());
    row.match_name = Some(component.name.clone());
    row.match_text = Some(component.signature.clone());
    row.match_signature = Some(component.signature.clone());
    row.match_path = Some(class.name.clone());
    row.match_line_start = Some(component.line_number);
    row.match_line_end = Some(component.line_number);
    row.match_owner_name = Some(class.name.clone());
    row.match_owner_path = Some(class.name.clone());
    row.match_class_path = Some(class.name.clone());
    row.match_top_level = Some(false);
    row.match_visibility = Some("public".to_string());
    row.match_type_name = Some(component.type_name.clone());
    row.match_variability.clone_from(&component.variability);
    row.match_direction.clone_from(&component.direction);
    row.match_component_kind = Some(component.component_kind.clone());
    row.match_default_value.clone_from(&component.default_value);
    row.match_is_partial = Some(false);
    row.match_is_final = Some(false);
    row.match_is_encapsulated = Some(false);
    row
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
            optional_utf8(rows, |row| row.item_dependency_form.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_target.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_alias.as_deref()),
            optional_utf8(rows, |row| row.item_dependency_local_name.as_deref()),
            optional_utf8(rows, |row| row.item_text.as_deref()),
            optional_i64(rows, |row| row.item_line_start),
            optional_i64(rows, |row| row.item_line_end),
            optional_utf8(rows, |row| row.item_owner_name.as_deref()),
            optional_utf8(rows, |row| row.item_owner_path.as_deref()),
            optional_utf8(rows, |row| row.item_visibility.as_deref()),
            optional_utf8(rows, |row| row.item_type_name.as_deref()),
            optional_utf8(rows, |row| row.item_variability.as_deref()),
            optional_utf8(rows, |row| row.item_direction.as_deref()),
            optional_utf8(rows, |row| row.item_component_kind.as_deref()),
            optional_utf8(rows, |row| row.item_array_dimensions.as_deref()),
            optional_utf8(rows, |row| row.item_default_value.as_deref()),
            optional_utf8(rows, |row| row.item_start_value.as_deref()),
            optional_utf8(rows, |row| row.item_modifier_names.as_deref()),
            optional_utf8(rows, |row| row.item_unit.as_deref()),
            optional_utf8(rows, |row| row.item_class_path.as_deref()),
            optional_bool(rows, |row| row.item_top_level),
            optional_bool(rows, |row| row.item_is_partial),
            optional_bool(rows, |row| row.item_is_final),
            optional_bool(rows, |row| row.item_is_encapsulated),
        ],
    )
}

fn ast_query_rows_to_batch(rows: &[AstQueryRow]) -> Result<RecordBatch, arrow::error::ArrowError> {
    RecordBatch::try_new(
        ast_query_response_schema(),
        vec![
            ast_required_utf8(rows, |row| &row.request_id),
            ast_required_utf8(rows, |row| &row.source_id),
            ast_repeated_utf8(rows, AST_QUERY_SUMMARY_KIND),
            ast_repeated_utf8(rows, AST_QUERY_BACKEND),
            ast_required_bool(rows, |_| true),
            ast_optional_utf8(rows, |row| row.primary_name.as_deref()),
            ast_optional_i64(rows, |row| row.match_count),
            ast_null_utf8(rows),
            ast_optional_i64(rows, |row| row.match_index),
            ast_optional_utf8(rows, |row| row.match_node_kind.as_deref()),
            ast_optional_utf8(rows, |row| row.match_name.as_deref()),
            ast_optional_utf8(rows, |row| row.match_text.as_deref()),
            ast_optional_utf8(rows, |row| row.match_signature.as_deref()),
            ast_optional_utf8(rows, |row| row.match_path.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_kind.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_form.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_target.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_local_name.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_parent.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_member.as_deref()),
            ast_optional_utf8(rows, |row| row.match_dependency_alias.as_deref()),
            ast_optional_i64(rows, |row| row.match_line_start),
            ast_optional_i64(rows, |row| row.match_line_end),
            ast_optional_utf8(rows, |row| row.match_owner_name.as_deref()),
            ast_optional_utf8(rows, |row| row.match_owner_path.as_deref()),
            ast_optional_utf8(rows, |row| row.match_class_path.as_deref()),
            ast_optional_bool(rows, |row| row.match_top_level),
            ast_optional_utf8(rows, |row| row.match_visibility.as_deref()),
            ast_optional_utf8(rows, |row| row.match_type_name.as_deref()),
            ast_optional_utf8(rows, |row| row.match_variability.as_deref()),
            ast_optional_utf8(rows, |row| row.match_direction.as_deref()),
            ast_optional_utf8(rows, |row| row.match_component_kind.as_deref()),
            ast_optional_utf8(rows, |row| row.match_array_dimensions.as_deref()),
            ast_optional_utf8(rows, |row| row.match_default_value.as_deref()),
            ast_optional_utf8(rows, |row| row.match_start_value.as_deref()),
            ast_optional_utf8(rows, |row| row.match_modifier_names.as_deref()),
            ast_optional_utf8(rows, |row| row.match_unit.as_deref()),
            ast_optional_bool(rows, |row| row.match_is_partial),
            ast_optional_bool(rows, |row| row.match_is_final),
            ast_optional_bool(rows, |row| row.match_is_encapsulated),
        ],
    )
}

fn ast_query_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        utf8("request_id", false),
        utf8("source_id", false),
        utf8("summary_kind", false),
        utf8("backend", false),
        bool_field("success", false),
        utf8("primary_name", true),
        int64("match_count"),
        utf8("error_message", true),
        int64("match_index"),
        utf8("match_node_kind", true),
        utf8("match_name", true),
        utf8("match_text", true),
        utf8("match_signature", true),
        utf8("match_path", true),
        utf8("match_dependency_kind", true),
        utf8("match_dependency_form", true),
        utf8("match_dependency_target", true),
        utf8("match_dependency_local_name", true),
        utf8("match_dependency_parent", true),
        utf8("match_dependency_member", true),
        utf8("match_dependency_alias", true),
        int64("match_line_start"),
        int64("match_line_end"),
        utf8("match_owner_name", true),
        utf8("match_owner_path", true),
        utf8("match_class_path", true),
        bool_field("match_top_level", true),
        utf8("match_visibility", true),
        utf8("match_type_name", true),
        utf8("match_variability", true),
        utf8("match_direction", true),
        utf8("match_component_kind", true),
        utf8("match_array_dimensions", true),
        utf8("match_default_value", true),
        utf8("match_start_value", true),
        utf8("match_modifier_names", true),
        utf8("match_unit", true),
        bool_field("match_is_partial", true),
        bool_field("match_is_final", true),
        bool_field("match_is_encapsulated", true),
    ]))
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

fn ast_required_utf8(rows: &[AstQueryRow], value: impl Fn(&AstQueryRow) -> &str) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn ast_repeated_utf8(rows: &[AstQueryRow], value: &str) -> ArrayRef {
    Arc::new(StringArray::from(vec![value; rows.len()]))
}

fn ast_optional_utf8<'a>(
    rows: &'a [AstQueryRow],
    value: impl Fn(&'a AstQueryRow) -> Option<&'a str>,
) -> ArrayRef {
    Arc::new(StringArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn ast_null_utf8(rows: &[AstQueryRow]) -> ArrayRef {
    Arc::new(StringArray::from(vec![None::<&str>; rows.len()]))
}

fn ast_required_bool(rows: &[AstQueryRow], value: impl Fn(&AstQueryRow) -> bool) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn ast_optional_bool(
    rows: &[AstQueryRow],
    value: impl Fn(&AstQueryRow) -> Option<bool>,
) -> ArrayRef {
    Arc::new(BooleanArray::from(
        rows.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn ast_optional_i64(rows: &[AstQueryRow], value: impl Fn(&AstQueryRow) -> Option<i64>) -> ArrayRef {
    Arc::new(Int64Array::from(rows.iter().map(value).collect::<Vec<_>>()))
}
