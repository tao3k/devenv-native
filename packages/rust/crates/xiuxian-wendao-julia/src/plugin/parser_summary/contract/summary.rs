//! Julia parser-summary file and root summary projection.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::JuliaParserSummaryResponseRow;
use super::values::parser_summary_contract_error;
use crate::plugin::parser_summary::route::ParserSummaryRouteKind;
use crate::plugin::parser_summary::types::{
    JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserFileSummary, JuliaParserImport,
    JuliaParserSourceSummary, JuliaParserSymbol, JuliaParserSymbolKind,
};

pub(crate) fn decode_julia_parser_file_summary(
    route_kind: ParserSummaryRouteKind,
    rows: &[JuliaParserSummaryResponseRow],
) -> Result<JuliaParserFileSummary, RepoIntelligenceError> {
    let summary_context = response_context(route_kind, rows)?;
    let exports = collect_exports(rows);
    let import_map = collect_import_map(rows);
    let includes = collect_includes(rows);
    let symbol_map = collect_symbol_map(rows)?;
    let docstrings = collect_docstrings(rows)?;

    Ok(JuliaParserFileSummary {
        module_name: summary_context.module_name.clone(),
        exports,
        imports: import_map.into_values().collect(),
        symbols: symbol_map.into_values().collect(),
        docstrings,
        includes,
    })
}

pub(crate) fn decode_julia_parser_root_summary(
    route_kind: ParserSummaryRouteKind,
    rows: &[JuliaParserSummaryResponseRow],
) -> Result<JuliaParserSourceSummary, RepoIntelligenceError> {
    let summary = decode_julia_parser_file_summary(route_kind, rows)?;
    let Some(module_name) = summary.module_name else {
        return Err(parser_summary_contract_error(
            "response",
            format!(
                "Julia parser-summary route `{}` did not return `module_name`",
                route_kind.summary_kind(),
            ),
        ));
    };
    Ok(JuliaParserSourceSummary {
        module_name,
        exports: summary.exports,
        imports: summary.imports,
        symbols: summary.symbols,
        docstrings: summary.docstrings,
        includes: summary.includes,
    })
}

fn collect_exports(rows: &[JuliaParserSummaryResponseRow]) -> Vec<String> {
    rows.iter()
        .filter(|row| row.item_group.as_deref() == Some("export"))
        .filter_map(|row| row.item_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_import_map(
    rows: &[JuliaParserSummaryResponseRow],
) -> BTreeMap<String, JuliaParserImport> {
    let mut import_map = BTreeMap::<String, JuliaParserImport>::new();
    for row in rows
        .iter()
        .filter(|row| row.item_group.as_deref() == Some("import"))
    {
        let Some(module) = row
            .item_dependency_target
            .clone()
            .or_else(|| row.item_name.clone())
        else {
            continue;
        };
        let candidate = JuliaParserImport {
            module: module.clone(),
            reexported: row.item_reexported.unwrap_or(false),
            dependency_kind: row
                .item_dependency_kind
                .clone()
                .unwrap_or_else(|| "import".to_string()),
            dependency_form: row
                .item_dependency_form
                .clone()
                .unwrap_or_else(|| "path".to_string()),
            dependency_is_relative: row.item_dependency_is_relative.unwrap_or(false),
            dependency_relative_level: row.item_dependency_relative_level.unwrap_or(0),
            dependency_local_name: row.item_dependency_local_name.clone(),
            dependency_parent: row.item_dependency_parent.clone(),
            dependency_member: row.item_dependency_member.clone(),
            dependency_alias: row.item_dependency_alias.clone(),
        };
        match import_map.get(&module) {
            Some(existing) if existing.reexported || !candidate.reexported => {}
            _ => {
                import_map.insert(module, candidate);
            }
        }
    }
    import_map
}

fn collect_includes(rows: &[JuliaParserSummaryResponseRow]) -> Vec<String> {
    rows.iter()
        .filter(|row| row.item_group.as_deref() == Some("include"))
        .filter_map(|row| {
            row.item_path
                .clone()
                .or_else(|| row.item_dependency_target.clone())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_symbol_map(
    rows: &[JuliaParserSummaryResponseRow],
) -> Result<BTreeMap<String, JuliaParserSymbol>, RepoIntelligenceError> {
    let mut symbol_map = BTreeMap::<String, JuliaParserSymbol>::new();
    for row in rows
        .iter()
        .filter(|row| row.item_group.as_deref() == Some("symbol"))
    {
        let Some(name) = row.item_name.clone() else {
            continue;
        };
        let symbol = JuliaParserSymbol {
            name: name.clone(),
            kind: map_symbol_kind(row.item_kind.as_deref(), row.item_binding_kind.as_deref()),
            signature: row.item_signature.clone(),
            line_start: normalize_line_number(row.item_line_start, "item_line_start")?,
            line_end: normalize_line_number(row.item_line_end, "item_line_end")?,
            attributes: build_symbol_attributes(row),
        };
        match symbol_map.get(&name) {
            Some(existing) if symbol_kind_rank(existing.kind) > symbol_kind_rank(symbol.kind) => {}
            _ => {
                symbol_map.insert(name, symbol);
            }
        }
    }
    Ok(symbol_map)
}

fn collect_docstrings(
    rows: &[JuliaParserSummaryResponseRow],
) -> Result<Vec<JuliaParserDocAttachment>, RepoIntelligenceError> {
    rows.iter()
        .filter(|row| row.item_group.as_deref() == Some("docstring"))
        .filter(|row| {
            row.item_target_name.is_some() || row.item_name.is_some() && row.item_content.is_some()
        })
        .map(|row| {
            let target_name = row
                .item_target_name
                .as_ref()
                .or(row.item_name.as_ref())
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        "response",
                        "parser-summary docstring row is missing target name",
                    )
                })?
                .clone();
            let doc_content = row.item_content.as_ref().ok_or_else(|| {
                parser_summary_contract_error(
                    "response",
                    format!(
                        "parser-summary docstring row for `{target_name}` is missing `item_content`"
                    ),
                )
            })?;
            Ok(JuliaParserDocAttachment {
                target_name,
                target_kind: map_doc_target_kind(row.item_target_kind.as_deref()),
                target_path: row.item_target_path.clone(),
                target_line_start: normalize_line_number(
                    row.item_target_line_start,
                    "item_target_line_start",
                )?,
                target_line_end: normalize_line_number(
                    row.item_target_line_end,
                    "item_target_line_end",
                )?,
                content: doc_content.clone(),
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()
        .map(|docstrings| docstrings.into_iter().collect())
}

fn build_symbol_attributes(row: &JuliaParserSummaryResponseRow) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();

    insert_text_attribute(&mut attributes, "parser_kind", row.item_kind.as_ref());
    insert_text_attribute(&mut attributes, "module_kind", row.module_kind.as_ref());
    insert_text_attribute(
        &mut attributes,
        "binding_kind",
        row.item_binding_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "module_name",
        row.item_module_name.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "module_path",
        row.item_module_path.as_ref(),
    );
    insert_text_attribute(&mut attributes, "owner_name", row.item_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_kind", row.item_owner_kind.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.item_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "type_kind", row.item_type_kind.as_ref());
    insert_text_attribute(
        &mut attributes,
        "type_parameters",
        row.item_type_parameters.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "type_supertype",
        row.item_type_supertype.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "parameter_kind",
        row.item_parameter_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "parameter_type_name",
        row.item_parameter_type_name.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "parameter_default_value",
        row.item_parameter_default_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "function_where_params",
        row.item_function_where_params.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "function_return_type",
        row.item_function_return_type.as_ref(),
    );
    insert_bool_attribute(&mut attributes, "top_level", row.item_top_level);
    insert_bool_attribute(
        &mut attributes,
        "parameter_is_typed",
        row.item_parameter_is_typed,
    );
    insert_bool_attribute(
        &mut attributes,
        "parameter_is_defaulted",
        row.item_parameter_is_defaulted,
    );
    insert_bool_attribute(
        &mut attributes,
        "parameter_is_vararg",
        row.item_parameter_is_vararg,
    );
    insert_bool_attribute(
        &mut attributes,
        "function_has_varargs",
        row.item_function_has_varargs,
    );
    insert_int_attribute(
        &mut attributes,
        "primitive_bits",
        row.item_primitive_bits.map(i64::from),
    );
    insert_int_attribute(
        &mut attributes,
        "function_positional_arity",
        row.item_function_positional_arity.map(i64::from),
    );
    insert_int_attribute(
        &mut attributes,
        "function_keyword_arity",
        row.item_function_keyword_arity.map(i64::from),
    );

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

fn insert_int_attribute(attributes: &mut BTreeMap<String, String>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        attributes.insert(key.to_string(), value.to_string());
    }
}

fn normalize_line_number(
    value: Option<i64>,
    field_name: &str,
) -> Result<Option<usize>, RepoIntelligenceError> {
    value
        .map(|value| {
            usize::try_from(value).map_err(|error| {
                parser_summary_contract_error(
                    "response",
                    format!(
                        "parser-summary column `{field_name}` cannot narrow `{value}` into usize: {error}"
                    ),
                )
            })
        })
        .transpose()
}

fn response_context(
    route_kind: ParserSummaryRouteKind,
    rows: &[JuliaParserSummaryResponseRow],
) -> Result<&JuliaParserSummaryResponseRow, RepoIntelligenceError> {
    let Some(first) = rows.first() else {
        return Err(parser_summary_contract_error(
            "response",
            "parser-summary response rows must contain at least one row".to_string(),
        ));
    };
    let expected_summary_kind = route_kind.summary_kind();
    for row in rows {
        if row.summary_kind != expected_summary_kind {
            return Err(parser_summary_contract_error(
                "response",
                format!(
                    "parser-summary response row for request `{}` returned summary kind `{}` but expected `{expected_summary_kind}`",
                    row.request_id, row.summary_kind,
                ),
            ));
        }
        if row.request_id != first.request_id {
            return Err(parser_summary_contract_error(
                "response",
                "parser-summary response rows must not mix request ids".to_string(),
            ));
        }
        if row.source_id != first.source_id {
            return Err(parser_summary_contract_error(
                "response",
                "parser-summary response rows must not mix source ids".to_string(),
            ));
        }
        if row.success != first.success {
            return Err(parser_summary_contract_error(
                "response",
                "parser-summary response rows must agree on `success`".to_string(),
            ));
        }
    }
    if !first.success {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "Julia parser-summary route `{}` failed for source `{}`: {}",
                route_kind.summary_kind(),
                first.source_id,
                first
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "unknown parser error".to_string()),
            ),
        });
    }
    Ok(first)
}

fn map_symbol_kind(kind: Option<&str>, binding_kind: Option<&str>) -> JuliaParserSymbolKind {
    match (kind, binding_kind) {
        (Some("function"), _) => JuliaParserSymbolKind::Function,
        (Some("type"), _) => JuliaParserSymbolKind::Type,
        (Some("binding"), Some("const")) => JuliaParserSymbolKind::Constant,
        _ => JuliaParserSymbolKind::Other,
    }
}

fn symbol_kind_rank(kind: JuliaParserSymbolKind) -> u8 {
    match kind {
        JuliaParserSymbolKind::Type => 3,
        JuliaParserSymbolKind::Constant => 2,
        JuliaParserSymbolKind::Function => 1,
        JuliaParserSymbolKind::Other => 0,
    }
}

fn map_doc_target_kind(target_kind: Option<&str>) -> JuliaParserDocTargetKind {
    match target_kind {
        Some("module") => JuliaParserDocTargetKind::Module,
        _ => JuliaParserDocTargetKind::Symbol,
    }
}
