//! Modelica AST-query repository-analysis projection.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

use xiuxian_wendao_core::repo_intelligence::{
    ImportKind, ModuleRecord, RepoIntelligenceError, RepoSymbolKind, RepositoryAnalysisOutput,
    SymbolRecord,
};

use super::ModelicaAstQueryResponseRow;
use super::values::ast_query_contract_error;

pub(crate) fn decode_modelica_ast_query_analysis(
    repo_id: &str,
    source_id: &str,
    rows: &[ModelicaAstQueryResponseRow],
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let first_row = validate_modelica_ast_query_analysis_rows(rows)?;
    let (module_id, mut output) =
        initialize_modelica_ast_query_analysis_output(repo_id, source_id, first_row);
    let mut seen_symbols = BTreeSet::<(String, String, String)>::new();
    let mut seen_imports = BTreeSet::<(String, String, String, String)>::new();
    for row in rows {
        append_modelica_ast_query_analysis_row(
            repo_id,
            source_id,
            &module_id,
            row,
            &mut output,
            &mut seen_symbols,
            &mut seen_imports,
        )?;
    }

    Ok(output)
}

fn validate_modelica_ast_query_analysis_rows(
    rows: &[ModelicaAstQueryResponseRow],
) -> Result<&ModelicaAstQueryResponseRow, RepoIntelligenceError> {
    let Some(first_row) = rows.first() else {
        return Err(ast_query_contract_error(
            "response",
            "ast-query response contained no rows",
        ));
    };
    if first_row.summary_kind != "modelica_ast_query" {
        return Err(ast_query_contract_error(
            "response",
            format!(
                "expected `modelica_ast_query` summary kind, found `{}`",
                first_row.summary_kind
            ),
        ));
    }
    if first_row.backend != "OMParser.jl" {
        return Err(ast_query_contract_error(
            "response",
            format!(
                "expected `OMParser.jl` backend, found `{}`",
                first_row.backend
            ),
        ));
    }
    if rows.iter().any(|row| !row.success) {
        let message = rows
            .iter()
            .find_map(|row| row.error_message.clone())
            .unwrap_or_else(|| "unknown Modelica AST query failure".to_string());
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!("Modelica AST query request failed: {message}"),
        });
    }
    Ok(first_row)
}

fn initialize_modelica_ast_query_analysis_output(
    repo_id: &str,
    source_id: &str,
    first_row: &ModelicaAstQueryResponseRow,
) -> (String, RepositoryAnalysisOutput) {
    let primary_name = first_row
        .primary_name
        .clone()
        .or_else(|| {
            std::path::Path::new(source_id)
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "modelica".to_string());
    let module_id = format!("repo:{repo_id}:module:{primary_name}");
    let output = RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: (repo_id.to_string()).into(),
            module_id: (module_id.clone()).into(),
            qualified_name: primary_name,
            path: (source_id.to_string()).into(),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    (module_id, output)
}

fn append_modelica_ast_query_analysis_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_symbols: &mut BTreeSet<(String, String, String)>,
    seen_imports: &mut BTreeSet<(String, String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(node_kind) = row.match_node_kind.as_deref() else {
        return Ok(());
    };
    if matches!(node_kind, "import" | "extends") {
        append_modelica_ast_query_import_row(
            repo_id,
            source_id,
            module_id,
            row,
            output,
            seen_imports,
        )?;
        return Ok(());
    }
    append_modelica_ast_query_symbol_row(
        repo_id,
        source_id,
        module_id,
        node_kind,
        row,
        output,
        seen_symbols,
    )
}

fn append_modelica_ast_query_import_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_imports: &mut BTreeSet<(String, String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(import_record) = import_record_from_ast_row(repo_id, source_id, module_id, row)?
    else {
        return Ok(());
    };
    let key = (
        import_record.source_module.clone(),
        import_record.import_name.clone(),
        import_record.path.to_string(),
        import_record
            .attributes
            .get("dependency_form")
            .cloned()
            .unwrap_or_default(),
    );
    if seen_imports.insert(key) {
        output.imports.push(import_record);
    }
    Ok(())
}

fn append_modelica_ast_query_symbol_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    node_kind: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_symbols: &mut BTreeSet<(String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(kind) = ast_row_symbol_kind(row) else {
        return Ok(());
    };
    let name = row.match_name.clone().unwrap_or_default();
    if name.trim().is_empty() {
        return Ok(());
    }
    let owner_path = row.match_owner_path.clone().unwrap_or_default();
    let symbol_key = (name.clone(), node_kind.to_string(), owner_path.clone());
    if !seen_symbols.insert(symbol_key) {
        return Ok(());
    }

    let qualified_name = ast_row_qualified_name(row, &name);
    output.symbols.push(SymbolRecord {
        repo_id: (repo_id.to_string()).into(),
        symbol_id: (format!("repo:{repo_id}:symbol:{qualified_name}")).into(),
        module_id: Some((module_id.to_string()).into()),
        name,
        qualified_name,
        kind,
        path: (source_id.to_string()).into(),
        line_start: ast_line_number(row.match_line_start)?,
        line_end: ast_line_number(row.match_line_end)?,
        signature: row
            .match_signature
            .clone()
            .or_else(|| row.match_text.clone())
            .or_else(|| row.match_name.clone()),
        audit_status: None,
        verification_state: None,
        attributes: ast_row_symbol_attributes(row),
    });
    Ok(())
}

fn ast_row_symbol_kind(row: &ModelicaAstQueryResponseRow) -> Option<RepoSymbolKind> {
    match row.match_node_kind.as_deref() {
        Some("function") => Some(RepoSymbolKind::Function),
        Some(
            "package"
            | "model"
            | "record"
            | "block"
            | "connector"
            | "expandable_connector"
            | "type"
            | "enumeration"
            | "operator"
            | "operator_record"
            | "uniontype"
            | "metarecord"
            | "class",
        ) => Some(RepoSymbolKind::Type),
        Some("component") => match row.match_component_kind.as_deref() {
            Some("constant" | "parameter") => Some(RepoSymbolKind::Constant),
            _ => Some(RepoSymbolKind::Other),
        },
        _ => None,
    }
}

fn import_record_from_ast_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
) -> Result<Option<xiuxian_wendao_core::repo_intelligence::ImportRecord>, RepoIntelligenceError> {
    let source_module = row
        .match_dependency_target
        .clone()
        .or_else(|| row.match_name.clone())
        .unwrap_or_default();
    if source_module.trim().is_empty() {
        return Ok(None);
    }
    let import_name = row
        .match_dependency_local_name
        .clone()
        .or_else(|| row.match_dependency_alias.clone())
        .or_else(|| source_module.rsplit('.').next().map(str::to_string))
        .unwrap_or_else(|| source_module.clone());
    let target_package = source_module
        .split('.')
        .next()
        .unwrap_or(source_module.as_str())
        .to_string();

    let mut attributes = BTreeMap::new();
    insert_text_attribute(
        &mut attributes,
        "dependency_kind",
        row.match_dependency_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_form",
        row.match_dependency_form.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_target",
        row.match_dependency_target.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_alias",
        row.match_dependency_alias.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_local_name",
        row.match_dependency_local_name.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_parent",
        row.match_dependency_parent.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_member",
        row.match_dependency_member.as_ref(),
    );
    insert_text_attribute(&mut attributes, "owner_name", row.match_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.match_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "class_path", row.match_class_path.as_ref());

    Ok(Some(xiuxian_wendao_core::repo_intelligence::ImportRecord {
        repo_id: (repo_id.to_string()).into(),
        module_id: (module_id.to_string()).into(),
        path: (source_id.to_string()).into(),
        import_name,
        target_package,
        source_module,
        kind: ast_row_import_kind(row),
        line_start: ast_line_number(row.match_line_start)?,
        resolved_id: None,
        attributes,
    }))
}

fn ast_row_import_kind(row: &ModelicaAstQueryResponseRow) -> ImportKind {
    match row.match_dependency_form.as_deref() {
        Some("named_import" | "unqualified_import" | "group_import" | "extends") => {
            ImportKind::Module
        }
        _ => ImportKind::Symbol,
    }
}

fn ast_row_qualified_name(row: &ModelicaAstQueryResponseRow, name: &str) -> String {
    if let Some(class_path) = row
        .match_class_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if class_path == name {
            return class_path.to_string();
        }
        return format!("{class_path}.{name}");
    }
    if let Some(owner_path) = row
        .match_owner_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return format!("{owner_path}.{name}");
    }
    name.to_string()
}

fn ast_row_symbol_attributes(row: &ModelicaAstQueryResponseRow) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();
    insert_text_attribute(&mut attributes, "parser_kind", row.match_node_kind.as_ref());
    if let Some(restriction) = row.match_node_kind.as_deref().filter(|value| {
        matches!(
            *value,
            "package"
                | "model"
                | "record"
                | "block"
                | "connector"
                | "expandable_connector"
                | "type"
                | "enumeration"
                | "operator"
                | "operator_record"
                | "uniontype"
                | "metarecord"
                | "class"
        )
    }) {
        attributes.insert("restriction".to_string(), restriction.to_string());
    }
    insert_text_attribute(&mut attributes, "visibility", row.match_visibility.as_ref());
    insert_text_attribute(&mut attributes, "type_name", row.match_type_name.as_ref());
    insert_text_attribute(
        &mut attributes,
        "variability",
        row.match_variability.as_ref(),
    );
    insert_text_attribute(&mut attributes, "direction", row.match_direction.as_ref());
    insert_text_attribute(
        &mut attributes,
        "component_kind",
        row.match_component_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "array_dimensions",
        row.match_array_dimensions.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "default_value",
        row.match_default_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "start_value",
        row.match_start_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "modifier_names",
        row.match_modifier_names.as_ref(),
    );
    insert_text_attribute(&mut attributes, "unit", row.match_unit.as_ref());
    insert_text_attribute(&mut attributes, "owner_name", row.match_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.match_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "class_path", row.match_class_path.as_ref());
    insert_bool_attribute(&mut attributes, "top_level", row.match_top_level);
    insert_bool_attribute(&mut attributes, "is_partial", row.match_is_partial);
    insert_bool_attribute(&mut attributes, "is_final", row.match_is_final);
    insert_bool_attribute(
        &mut attributes,
        "is_encapsulated",
        row.match_is_encapsulated,
    );
    attributes
}

fn ast_line_number(value: Option<i64>) -> Result<Option<usize>, RepoIntelligenceError> {
    value
        .map(usize::try_from)
        .transpose()
        .map_err(|error| ast_query_contract_error("response", error.to_string()))
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
