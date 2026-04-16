use crate::analyzers::{ImportKind, ImportRecord, RepoSymbolKind, RepositoryAnalysisOutput};
use crate::gateway::studio::types::{
    CodeAstAnalysisResponse, CodeAstEdge, CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind,
    CodeAstProjection, CodeAstProjectionKind, CodeAstRetrievalAtomScope,
};

use super::{
    RetrievalChunkLineExt, build_code_ast_retrieval_atom, build_code_block_retrieval_atoms,
    focus_symbol_for_blocks, path_has_extension, repo_relative_path_matches,
    retrieval_semantic_type,
};

/// Build the code-AST response payload for one repository-relative source path.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_code_ast_analysis_response(
    repo_id: String,
    path: String,
    line_hint: Option<usize>,
    source_content: Option<&str>,
    analysis: &RepositoryAnalysisOutput,
) -> CodeAstAnalysisResponse {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut retrieval_atoms = Vec::new();
    let mut contains_edge_count = 0usize;
    let mut uses_edge_count = 0usize;
    let mut interaction_edge_count = 0usize;

    // Convert modules to nodes
    for module in &analysis.modules {
        let line = 1usize;
        nodes.push(CodeAstNode {
            id: module.module_id.clone(),
            label: module.qualified_name.clone(),
            kind: CodeAstNodeKind::Module,
            path: Some(module.path.clone()),
            line_start: Some(line),
            line_end: Some(line),
        });
        let content = format!("{}|{}", module.qualified_name, module.path);
        let declaration_locator = format!("l{line}");
        let symbol_locator = format!("{}-l{line}", module.qualified_name);
        retrieval_atoms.push(
            build_code_ast_retrieval_atom(
                module.module_id.as_str(),
                module.path.as_str(),
                CodeAstRetrievalAtomScope::Declaration,
                "module",
                declaration_locator.as_str(),
                content.as_str(),
            )
            .with_lines(line, line)
            .with_display(
                format!("Declaration Rail · {}", module.qualified_name),
                format!("module {}", module.qualified_name),
            ),
        );
        retrieval_atoms.push(
            build_code_ast_retrieval_atom(
                module.module_id.as_str(),
                module.path.as_str(),
                CodeAstRetrievalAtomScope::Symbol,
                "module",
                symbol_locator.as_str(),
                content.as_str(),
            )
            .with_lines(line, line)
            .with_display(
                format!("Symbol Rail · {}", module.qualified_name),
                module.qualified_name.clone(),
            ),
        );
    }

    // Convert symbols to nodes
    for symbol in &analysis.symbols {
        let same_file = repo_relative_path_matches(symbol.path.as_str(), path.as_str());
        let kind = if same_file {
            match symbol.kind {
                RepoSymbolKind::Function => CodeAstNodeKind::Function,
                RepoSymbolKind::Type => CodeAstNodeKind::Type,
                RepoSymbolKind::Constant => CodeAstNodeKind::Constant,
                _ => CodeAstNodeKind::Other,
            }
        } else {
            CodeAstNodeKind::ExternalSymbol
        };
        nodes.push(CodeAstNode {
            id: symbol.symbol_id.clone(),
            label: symbol.name.clone(),
            kind,
            path: Some(symbol.path.clone()),
            line_start: symbol.line_start,
            line_end: symbol.line_end.or(symbol.line_start),
        });
        let semantic_type = retrieval_semantic_type(symbol, same_file);
        let declaration_locator = format!("l{}", symbol.line_start.unwrap_or(0));
        let symbol_locator = format!("{}-l{}", symbol.name, symbol.line_start.unwrap_or(0));
        let declaration_excerpt = symbol
            .signature
            .clone()
            .unwrap_or_else(|| symbol.name.clone());
        let symbol_attributes = build_symbol_retrieval_attributes(symbol);
        let content = format!(
            "{}|{}|{}|{}",
            symbol.qualified_name,
            symbol.path,
            semantic_type,
            symbol.signature.as_deref().unwrap_or(symbol.name.as_str())
        );

        if same_file {
            let mut declaration_atom = build_code_ast_retrieval_atom(
                symbol.symbol_id.as_str(),
                symbol.path.as_str(),
                CodeAstRetrievalAtomScope::Declaration,
                semantic_type,
                declaration_locator.as_str(),
                content.as_str(),
            )
            .with_display(
                format!("Declaration Rail · {}", symbol.name),
                declaration_excerpt.clone(),
            )
            .with_attributes(symbol_attributes.clone());
            if let Some(start) = symbol.line_start {
                declaration_atom =
                    declaration_atom.with_lines(start, symbol.line_end.unwrap_or(start));
            }
            retrieval_atoms.push(declaration_atom);
        }

        let mut symbol_atom = build_code_ast_retrieval_atom(
            symbol.symbol_id.as_str(),
            symbol.path.as_str(),
            CodeAstRetrievalAtomScope::Symbol,
            semantic_type,
            symbol_locator.as_str(),
            content.as_str(),
        )
        .with_display(
            format!("Symbol Rail · {}", symbol.name),
            symbol.name.clone(),
        )
        .with_attributes(symbol_attributes);
        if let Some(start) = symbol.line_start {
            symbol_atom = symbol_atom.with_lines(start, symbol.line_end.unwrap_or(start));
        }
        retrieval_atoms.push(symbol_atom);
    }

    let import_nodes =
        build_import_code_ast_nodes(repo_id.as_str(), path.as_str(), analysis.imports.as_slice());
    interaction_edge_count += import_nodes.len();
    for (node, edge, atom) in import_nodes {
        nodes.push(node);
        edges.push(edge);
        retrieval_atoms.push(atom);
    }

    if let Some(primary_symbol) = focus_symbol_for_blocks(line_hint, analysis, path.as_str())
        && let Some(content) = source_content
    {
        retrieval_atoms.extend(build_code_block_retrieval_atoms(
            path.as_str(),
            primary_symbol.line_start,
            content,
        ));
    }

    // Convert relations to edges
    for relation in &analysis.relations {
        let kind = match relation.kind {
            crate::analyzers::RelationKind::Contains => {
                contains_edge_count += 1;
                CodeAstEdgeKind::Contains
            }
            crate::analyzers::RelationKind::Calls => {
                interaction_edge_count += 1;
                CodeAstEdgeKind::Calls
            }
            crate::analyzers::RelationKind::Uses => {
                interaction_edge_count += 1;
                uses_edge_count += 1;
                CodeAstEdgeKind::Uses
            }
            crate::analyzers::RelationKind::Imports => {
                interaction_edge_count += 1;
                CodeAstEdgeKind::Imports
            }
            _ => CodeAstEdgeKind::Other,
        };
        edges.push(CodeAstEdge {
            id: format!(
                "{}-{}-{}",
                relation.source_id, relation.target_id, relation.kind as u8
            ),
            source_id: relation.source_id.clone(),
            target_id: relation.target_id.clone(),
            kind,
            label: None,
        });
    }

    let language = if path_has_extension(path.as_str(), "jl") {
        "julia"
    } else {
        "modelica"
    };
    let focus_node_id = if let Some(line) = line_hint {
        analysis
            .symbols
            .iter()
            .find(|symbol| {
                if !repo_relative_path_matches(symbol.path.as_str(), path.as_str()) {
                    return false;
                }
                match (symbol.line_start, symbol.line_end) {
                    (Some(start), Some(end)) => start <= line && line <= end,
                    (Some(start), None) => start == line,
                    _ => false,
                }
            })
            .map(|symbol| symbol.symbol_id.clone())
    } else {
        analysis
            .symbols
            .iter()
            .find(|symbol| repo_relative_path_matches(symbol.path.as_str(), path.as_str()))
            .map(|symbol| symbol.symbol_id.clone())
    };
    let projections = vec![
        CodeAstProjection {
            kind: CodeAstProjectionKind::Contains,
            node_count: nodes.len(),
            edge_count: contains_edge_count,
        },
        CodeAstProjection {
            kind: CodeAstProjectionKind::Calls,
            node_count: nodes.len(),
            edge_count: interaction_edge_count,
        },
        CodeAstProjection {
            kind: CodeAstProjectionKind::Uses,
            node_count: nodes.len(),
            edge_count: uses_edge_count,
        },
    ];

    CodeAstAnalysisResponse {
        repo_id,
        path,
        language: language.to_string(),
        node_count: nodes.len(),
        edge_count: edges.len(),
        nodes,
        edges,
        projections,
        retrieval_atoms,
        focus_node_id,
        diagnostics: Vec::new(),
    }
}

fn build_import_code_ast_nodes(
    repo_id: &str,
    path: &str,
    imports: &[ImportRecord],
) -> Vec<(
    CodeAstNode,
    CodeAstEdge,
    crate::gateway::studio::types::CodeAstRetrievalAtom,
)> {
    imports
        .iter()
        .enumerate()
        .map(|(index, import)| {
            let import_id = import_node_id(repo_id, import, index);
            let semantic_type = import_semantic_type(import);
            let content = format!(
                "{}|{}|{}|{}",
                import.import_name, import.source_module, import.target_package, semantic_type
            );
            let mut attributes = vec![
                ("import_name".to_string(), import.import_name.clone()),
                ("target_package".to_string(), import.target_package.clone()),
                ("source_module".to_string(), import.source_module.clone()),
                ("import_kind".to_string(), import_kind_label(import.kind)),
            ];
            attributes.extend(
                import
                    .attributes
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
            let mut atom = build_code_ast_retrieval_atom(
                import_id.as_str(),
                path,
                CodeAstRetrievalAtomScope::Symbol,
                semantic_type,
                import.source_module.as_str(),
                content.as_str(),
            )
            .with_display(
                format!("Import Rail · {}", import.import_name),
                import.source_module.clone(),
            )
            .with_attributes(attributes);
            if let Some(start) = import.line_start {
                atom = atom.with_lines(start, start);
            }
            (
                CodeAstNode {
                    id: import_id.clone(),
                    label: import.import_name.clone(),
                    kind: CodeAstNodeKind::ExternalSymbol,
                    path: Some(path.to_string()),
                    line_start: import.line_start,
                    line_end: import.line_start,
                },
                CodeAstEdge {
                    id: format!("{}-{}-imports", import.module_id, import_id),
                    source_id: import.module_id.clone(),
                    target_id: import_id,
                    kind: CodeAstEdgeKind::Imports,
                    label: None,
                },
                atom,
            )
        })
        .collect()
}

fn build_symbol_retrieval_attributes(
    symbol: &crate::analyzers::SymbolRecord,
) -> Vec<(String, String)> {
    let mut attributes = symbol
        .attributes
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();
    if let Some(signature) = symbol.signature.as_ref() {
        attributes.push(("signature".to_string(), signature.clone()));
    }
    attributes
}

fn import_node_id(repo_id: &str, import: &ImportRecord, ordinal: usize) -> String {
    let target = import
        .resolved_id
        .as_deref()
        .unwrap_or(import.source_module.as_str());
    format!(
        "repo:{repo_id}:import:{}:{}:{}",
        import.module_id, target, ordinal
    )
}

fn import_semantic_type(import: &ImportRecord) -> &'static str {
    match import.kind {
        ImportKind::Module => "importModule",
        ImportKind::Symbol => "import",
        ImportKind::Reexport => "reexport",
    }
}

fn import_kind_label(kind: ImportKind) -> String {
    match kind {
        ImportKind::Module => "module".to_string(),
        ImportKind::Symbol => "symbol".to_string(),
        ImportKind::Reexport => "reexport".to_string(),
    }
}
