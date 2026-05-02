//! Modelica parser-summary file-summary projection.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

use xiuxian_wendao_core::repo_intelligence::{ImportKind, RepoIntelligenceError, RepoSymbolKind};

use crate::modelica_plugin::types::{ParsedDeclaration, ParsedImport};

use super::ModelicaParserSummaryResponseRow;
use super::values::parser_summary_contract_error;
use crate::modelica_plugin::parser_summary::route::ParserSummaryRouteKind;
use crate::modelica_plugin::parser_summary::types::ModelicaParserFileSummary;

pub(crate) fn decode_modelica_parser_file_summary(
    route_kind: ParserSummaryRouteKind,
    rows: &[ModelicaParserSummaryResponseRow],
) -> Result<ModelicaParserFileSummary, RepoIntelligenceError> {
    let _summary_context = modelica_response_context(route_kind, rows)?;
    let class_name = rows.iter().find_map(|row| row.class_name.clone());
    let mut equations_by_owner = collect_equations_by_owner(rows);
    let imports = collect_modelica_imports(rows)?;
    let declarations = collect_modelica_declarations(rows, &mut equations_by_owner)?;

    Ok(ModelicaParserFileSummary {
        class_name,
        imports,
        declarations,
    })
}

fn modelica_response_context(
    route_kind: ParserSummaryRouteKind,
    rows: &[ModelicaParserSummaryResponseRow],
) -> Result<&ModelicaParserSummaryResponseRow, RepoIntelligenceError> {
    let Some(first) = rows.first() else {
        return Err(parser_summary_contract_error(
            "response",
            format!(
                "Modelica parser-summary response for route `{}` did not contain any rows",
                route_kind.route(),
            ),
        ));
    };
    let expected_summary_kind = "modelica_file_summary";
    for row in rows {
        if row.summary_kind != expected_summary_kind {
            return Err(parser_summary_contract_error(
                "response",
                format!(
                    "Modelica parser-summary route `{}` returned unexpected summary kind `{}`",
                    route_kind.route(),
                    row.summary_kind,
                ),
            ));
        }
        if !row.success {
            return Err(RepoIntelligenceError::AnalysisFailed {
                message: row
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Modelica parser-summary request failed".to_string()),
            });
        }
    }
    Ok(first)
}

fn collect_equations_by_owner(
    rows: &[ModelicaParserSummaryResponseRow],
) -> BTreeMap<String, Vec<String>> {
    let mut equations_by_owner = BTreeMap::<String, Vec<String>>::new();
    for row in rows
        .iter()
        .filter(|row| row.item_group.as_deref() == Some("equation"))
    {
        let Some(text) = row.item_text.clone() else {
            continue;
        };
        equations_by_owner
            .entry(modelica_owner_key(row))
            .or_default()
            .push(text);
    }
    equations_by_owner
}

fn collect_modelica_imports(
    rows: &[ModelicaParserSummaryResponseRow],
) -> Result<Vec<ParsedImport>, RepoIntelligenceError> {
    let mut imports = Vec::new();
    let mut seen_imports = BTreeSet::new();
    for row in rows
        .iter()
        .filter(|row| row.item_group.as_deref() == Some("import"))
    {
        let name = row
            .item_dependency_target
            .clone()
            .or_else(|| row.item_name.clone())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let alias = row.item_dependency_alias.clone();
        let key = (
            name.clone(),
            alias.clone().unwrap_or_default(),
            row.item_dependency_form.clone().unwrap_or_default(),
        );
        if !seen_imports.insert(key) {
            continue;
        }
        imports.push(ParsedImport {
            name,
            alias,
            kind: modelica_import_kind(row.item_dependency_form.as_deref()),
            line_start: modelica_line_number(row.item_line_start)?,
            attributes: build_import_attributes(row),
        });
    }
    Ok(imports)
}

fn collect_modelica_declarations(
    rows: &[ModelicaParserSummaryResponseRow],
    equations_by_owner: &mut BTreeMap<String, Vec<String>>,
) -> Result<Vec<ParsedDeclaration>, RepoIntelligenceError> {
    let mut declarations = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.item_group.as_deref() == Some("symbol"))
    {
        let Some(kind) = modelica_kind_to_repo_kind(row.item_kind.as_deref()) else {
            continue;
        };
        let name = row.item_name.clone().unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let mut attributes = build_declaration_attributes(row);
        let equations = equations_by_owner
            .remove(&modelica_owner_key(row))
            .unwrap_or_default();
        if !equations.is_empty() {
            attributes.insert("equation_latex".to_string(), equations.join("\n\n"));
        }
        declarations.push(ParsedDeclaration {
            name,
            kind,
            signature: row
                .item_signature
                .clone()
                .or_else(|| row.item_name.clone())
                .unwrap_or_default(),
            line_start: modelica_line_number(row.item_line_start)?,
            line_end: modelica_line_number(row.item_line_end)?,
            equations,
            attributes,
        });
    }
    Ok(declarations)
}

fn modelica_owner_key(row: &ModelicaParserSummaryResponseRow) -> String {
    row.item_owner_path
        .clone()
        .or_else(|| row.item_owner_name.clone())
        .unwrap_or_default()
}

fn modelica_line_number(value: Option<i64>) -> Result<Option<usize>, RepoIntelligenceError> {
    value
        .map(usize::try_from)
        .transpose()
        .map_err(|error| parser_summary_contract_error("response", error.to_string()))
}

fn modelica_import_kind(form: Option<&str>) -> ImportKind {
    match form {
        Some("named_import" | "unqualified_import" | "group_import") => ImportKind::Module,
        _ => ImportKind::Symbol,
    }
}

fn build_import_attributes(row: &ModelicaParserSummaryResponseRow) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();
    insert_text_attribute(
        &mut attributes,
        "dependency_form",
        row.item_dependency_form.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_alias",
        row.item_dependency_alias.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_local_name",
        row.item_dependency_local_name.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_target",
        row.item_dependency_target.as_ref(),
    );
    attributes
}

fn modelica_kind_to_repo_kind(kind: Option<&str>) -> Option<RepoSymbolKind> {
    match kind {
        Some("function") => Some(RepoSymbolKind::Function),
        Some("model" | "record" | "block" | "connector" | "type") => Some(RepoSymbolKind::Type),
        Some("constant" | "parameter") => Some(RepoSymbolKind::Constant),
        _ => None,
    }
}

fn build_declaration_attributes(
    row: &ModelicaParserSummaryResponseRow,
) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();
    insert_text_attribute(&mut attributes, "parser_kind", row.item_kind.as_ref());
    insert_text_attribute(&mut attributes, "class_name", row.class_name.as_ref());
    insert_text_attribute(&mut attributes, "restriction", row.restriction.as_ref());
    insert_text_attribute(&mut attributes, "visibility", row.item_visibility.as_ref());
    insert_text_attribute(&mut attributes, "type_name", row.item_type_name.as_ref());
    insert_text_attribute(
        &mut attributes,
        "variability",
        row.item_variability.as_ref(),
    );
    insert_text_attribute(&mut attributes, "direction", row.item_direction.as_ref());
    insert_text_attribute(
        &mut attributes,
        "component_kind",
        row.item_component_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "array_dimensions",
        row.item_array_dimensions.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "default_value",
        row.item_default_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "start_value",
        row.item_start_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "modifier_names",
        row.item_modifier_names.as_ref(),
    );
    insert_text_attribute(&mut attributes, "unit", row.item_unit.as_ref());
    insert_text_attribute(&mut attributes, "owner_name", row.item_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.item_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "class_path", row.item_class_path.as_ref());
    insert_bool_attribute(&mut attributes, "top_level", row.item_top_level);
    insert_bool_attribute(&mut attributes, "is_partial", row.item_is_partial);
    insert_bool_attribute(&mut attributes, "is_final", row.item_is_final);
    insert_bool_attribute(&mut attributes, "is_encapsulated", row.item_is_encapsulated);
    attributes
}

fn insert_text_attribute(
    attributes: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<&String>,
) {
    let Some(value) = value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    attributes.insert(key.to_string(), value.to_string());
}

fn insert_bool_attribute(
    attributes: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        attributes.insert(key.to_string(), value.to_string());
    }
}
