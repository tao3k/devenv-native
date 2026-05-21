use std::collections::BTreeMap;

use crate::contracts::{
    CodeAstAnalysisResponse, CodeAstEdgeKind, CodeAstNodeKind, CodeAstProjectionKind,
    CodeAstRetrievalAtomScope,
};
use crate::studio::router::build_code_ast_analysis_response;
use xiuxian_wendao::analyzers::{
    ImportKind, ImportRecord, ModuleRecord, RelationKind, RelationRecord, RepoSymbolKind,
    RepositoryAnalysisOutput, SymbolRecord,
};

#[test]
fn build_code_ast_analysis_response_emits_uses_projection_and_external_node() {
    let payload = build_code_ast_analysis_response(
        "sciml".to_string(),
        "src/BaseModelica.jl".to_string(),
        Some(7),
        Some(sample_source()),
        &sample_analysis(),
    );

    assert_code_ast_summary(&payload);
    assert_code_ast_retrieval_atoms(&payload);
}

#[test]
fn build_code_ast_analysis_response_does_not_fallback_to_first_symbol_when_line_hint_misses() {
    let payload = build_code_ast_analysis_response(
        "sciml".to_string(),
        "src/BaseModelica.jl".to_string(),
        Some(1),
        Some(sample_source()),
        &sample_analysis(),
    );

    assert!(payload.focus_node_id.is_none());
    assert!(
        !payload
            .retrieval_atoms
            .iter()
            .any(|atom| matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Block)))
    );
}

fn sample_analysis() -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: "sciml".to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string().into(),
        }],
        symbols: vec![
            SymbolRecord {
                repo_id: "sciml".to_string().into(),
                symbol_id: "symbol:reexport".to_string().into(),
                module_id: Some("module:BaseModelica".to_string().into()),
                name: "reexport".to_string(),
                qualified_name: "BaseModelica.reexport".to_string(),
                kind: RepoSymbolKind::Function,
                path: "src/BaseModelica.jl".to_string().into(),
                line_start: Some(7),
                line_end: Some(9),
                signature: Some("reexport(input)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::from([
                    ("parser_kind".to_string(), "function".to_string()),
                    ("function_positional_arity".to_string(), "1".to_string()),
                    ("function_return_type".to_string(), "Result".to_string()),
                ]),
            },
            SymbolRecord {
                repo_id: "sciml".to_string().into(),
                symbol_id: "symbol:GLOBAL_FLAG".to_string().into(),
                module_id: Some("module:BaseModelica".to_string().into()),
                name: "GLOBAL_FLAG".to_string(),
                qualified_name: "BaseModelica.GLOBAL_FLAG".to_string(),
                kind: RepoSymbolKind::Other,
                path: "src/BaseModelica.jl".to_string().into(),
                line_start: Some(4),
                line_end: Some(4),
                signature: Some("global GLOBAL_FLAG = true".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::from([
                    ("parser_kind".to_string(), "binding".to_string()),
                    ("binding_kind".to_string(), "global".to_string()),
                ]),
            },
            SymbolRecord {
                repo_id: "sciml".to_string().into(),
                symbol_id: "symbol:ModelicaSystem".to_string().into(),
                module_id: None,
                name: "ModelicaSystem".to_string(),
                qualified_name: "ModelicaSystem".to_string(),
                kind: RepoSymbolKind::Type,
                path: "src/modelica/system.jl".to_string().into(),
                line_start: Some(1),
                line_end: Some(3),
                signature: None,
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            },
        ],
        imports: vec![ImportRecord {
            repo_id: "sciml".to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            path: "src/BaseModelica.jl".to_string().into(),
            import_name: "LinearAlgebra".to_string(),
            target_package: "LinearAlgebra".to_string(),
            source_module: "LinearAlgebra".to_string(),
            kind: ImportKind::Module,
            line_start: Some(2),
            resolved_id: None,
            attributes: BTreeMap::from([(
                "dependency_form".to_string(),
                "qualified_import".to_string(),
            )]),
        }],
        relations: vec![RelationRecord {
            repo_id: "sciml".to_string().into(),
            source_id: "symbol:reexport".to_string(),
            target_id: "symbol:ModelicaSystem".to_string(),
            kind: RelationKind::Uses,
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn sample_source() -> &'static str {
    "module BaseModelica\n\
\n\
# prelude\n\
# prelude\n\
# prelude\n\
# prelude\n\
pub fn reexport(\n\
    input,\n\
) {\n\
    if isempty(input)\n\
        return Err(Empty)\n\
    end\n\
\n\
    let meta = parse(input)\n\
\n\
    return Ok(meta)\n\
}\n"
}

fn assert_code_ast_summary(payload: &CodeAstAnalysisResponse) {
    assert_eq!(payload.language, "julia");
    assert!(
        payload
            .nodes
            .iter()
            .any(|node| matches!(node.kind, CodeAstNodeKind::ExternalSymbol))
    );
    assert!(
        payload
            .edges
            .iter()
            .any(|edge| matches!(edge.kind, CodeAstEdgeKind::Uses))
    );
    assert!(
        payload
            .edges
            .iter()
            .any(|edge| matches!(edge.kind, CodeAstEdgeKind::Imports))
    );
    assert!(payload.projections.iter().any(|projection| {
        matches!(projection.kind, CodeAstProjectionKind::Calls) && projection.edge_count > 0
    }));
    assert!(payload.focus_node_id.is_some());
}

fn assert_code_ast_retrieval_atoms(payload: &CodeAstAnalysisResponse) {
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "symbol:reexport"
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Declaration))
            && atom
                .chunk_id
                .starts_with("ast:src-basemodelica-jl:declaration:function:")
            && atom.display_label.as_deref() == Some("Declaration Rail · reexport")
            && atom.excerpt.as_deref() == Some("reexport(input)")
            && atom
                .attributes
                .iter()
                .any(|(key, value)| key == "function_positional_arity" && value == "1")
            && atom.token_estimate > 0
    }));
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "symbol:GLOBAL_FLAG"
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Declaration))
            && atom.semantic_type == "binding"
            && atom
                .chunk_id
                .starts_with("ast:src-basemodelica-jl:declaration:binding:")
    }));
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "symbol:ModelicaSystem"
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Symbol))
            && atom
                .chunk_id
                .starts_with("ast:src-modelica-system-jl:symbol:externalsymbol:")
            && atom.fingerprint.starts_with("fp:")
    }));
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id
            .starts_with("repo:sciml:import:module:BaseModelica:LinearAlgebra:")
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Symbol))
            && atom.semantic_type == "importModule"
            && atom.display_label.as_deref() == Some("Import Rail · LinearAlgebra")
            && atom.excerpt.as_deref() == Some("LinearAlgebra")
            && atom.line_start == Some(2)
            && atom
                .attributes
                .iter()
                .any(|(key, value)| key == "target_package" && value == "LinearAlgebra")
            && atom
                .attributes
                .iter()
                .any(|(key, value)| key == "dependency_form" && value == "qualified_import")
    }));
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id.starts_with("block:validation:")
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Block))
            && atom.semantic_type == "validation"
            && atom.line_start.is_some()
            && atom.line_end >= atom.line_start
    }));
    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id.starts_with("block:return:")
            && matches!(atom.surface, Some(CodeAstRetrievalAtomScope::Block))
            && atom.semantic_type == "return"
            && atom.line_start.is_some()
            && atom.line_end >= atom.line_start
    }));
}
