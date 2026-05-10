use std::collections::HashSet;
use std::path::Path;

use xiuxian_code_intelligence::{
    CodeLanguageId, extract_code_structure_symbols_for_language_id, first_code_signature_line,
};

use crate::studio::types::{
    CodeAstAnalysisResponse, CodeAstNode, CodeAstNodeKind, CodeAstProjection,
    CodeAstProjectionKind, CodeAstRetrievalAtomScope,
};

use super::{
    RetrievalChunkLineExt, build_code_ast_retrieval_atom, build_code_block_retrieval_atoms,
};

struct GenericAstItem {
    id: String,
    label: String,
    kind: CodeAstNodeKind,
    signature: String,
    line_start: usize,
    line_end: usize,
}

pub(crate) fn build_generic_code_ast_analysis_response(
    repo_id: String,
    path: String,
    line_hint: Option<usize>,
    source_content: &str,
    language_id: CodeLanguageId,
) -> CodeAstAnalysisResponse {
    let items = extract_generic_ast_items(
        repo_id.as_str(),
        path.as_str(),
        source_content,
        &language_id,
    );
    let focus_item = focus_generic_ast_item(items.as_slice(), line_hint);
    let focus_node_id = focus_item.map(|item| item.id.clone());
    let mut nodes = Vec::with_capacity(items.len());
    let mut retrieval_atoms = Vec::with_capacity(items.len() * 2);

    for item in &items {
        nodes.push(CodeAstNode {
            id: item.id.clone().into(),
            label: item.label.clone(),
            kind: item.kind,
            path: Some(path.clone().into()),
            line_start: Some(item.line_start),
            line_end: Some(item.line_end),
        });

        let semantic_type = generic_ast_semantic_type(item.kind);
        let content = format!(
            "{}|{}|{}|{}",
            item.label, path, semantic_type, item.signature
        );
        let attributes = vec![
            ("analysis_mode".to_string(), "ast-grep".to_string()),
            ("language".to_string(), language_id.as_str().to_string()),
        ];
        retrieval_atoms.push(
            build_code_ast_retrieval_atom(
                item.id.as_str(),
                path.as_str(),
                CodeAstRetrievalAtomScope::Declaration,
                semantic_type,
                format!("l{}", item.line_start).as_str(),
                content.as_str(),
            )
            .with_lines(item.line_start, item.line_end)
            .with_display(
                format!("Declaration Rail · {}", item.label),
                item.signature.clone(),
            )
            .with_attributes(attributes.clone()),
        );
        retrieval_atoms.push(
            build_code_ast_retrieval_atom(
                item.id.as_str(),
                path.as_str(),
                CodeAstRetrievalAtomScope::Symbol,
                semantic_type,
                format!("{}-l{}", item.label, item.line_start).as_str(),
                content.as_str(),
            )
            .with_lines(item.line_start, item.line_end)
            .with_display(format!("Symbol Rail · {}", item.label), item.label.clone())
            .with_attributes(attributes),
        );
    }

    if supports_generic_code_blocks(&language_id)
        && let Some(focus_item) = focus_item
    {
        retrieval_atoms.extend(build_code_block_retrieval_atoms(
            path.as_str(),
            Some(focus_item.line_start),
            source_content,
        ));
    }

    CodeAstAnalysisResponse {
        repo_id: repo_id.into(),
        path: path.into(),
        language: language_id.as_str().to_string(),
        node_count: nodes.len(),
        edge_count: 0,
        projections: vec![
            CodeAstProjection {
                kind: CodeAstProjectionKind::Contains,
                node_count: nodes.len(),
                edge_count: 0,
            },
            CodeAstProjection {
                kind: CodeAstProjectionKind::Calls,
                node_count: nodes.len(),
                edge_count: 0,
            },
            CodeAstProjection {
                kind: CodeAstProjectionKind::Uses,
                node_count: nodes.len(),
                edge_count: 0,
            },
        ],
        nodes,
        edges: Vec::new(),
        retrieval_atoms,
        focus_node_id: focus_node_id.map(Into::into),
        diagnostics: Vec::new(),
    }
}

fn extract_generic_ast_items(
    repo_id: &str,
    path: &str,
    source_content: &str,
    language_id: &CodeLanguageId,
) -> Vec<GenericAstItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for symbol in extract_code_structure_symbols_for_language_id(source_content, language_id) {
        let label = symbol
            .captures
            .get("NAME")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| fallback_item_label(path, symbol.signature.as_str()));
        let dedupe_key = format!("{path}:{}:{}:{label}", symbol.line_start, symbol.line_end);
        if !seen.insert(dedupe_key) {
            continue;
        }

        items.push(GenericAstItem {
            id: format!(
                "repo:{repo_id}:generic_ast:{}:{}:{}:{}",
                path, symbol.line_start, symbol.line_end, label
            ),
            label,
            kind: infer_generic_ast_node_kind(language_id, symbol.signature.as_str()),
            signature: symbol.signature,
            line_start: symbol.line_start,
            line_end: symbol.line_end,
        });
    }

    if !items.is_empty() {
        return items;
    }

    let label = fallback_item_label(path, source_content);
    let signature = first_signature_line(source_content).to_string();
    items.push(GenericAstItem {
        id: format!("repo:{repo_id}:generic_ast:{path}:1:1:{label}"),
        label,
        kind: if language_id.as_str() == "toml" {
            CodeAstNodeKind::Module
        } else {
            CodeAstNodeKind::Other
        },
        signature,
        line_start: 1,
        line_end: 1,
    });
    items
}

fn focus_generic_ast_item(
    items: &[GenericAstItem],
    line_hint: Option<usize>,
) -> Option<&GenericAstItem> {
    if let Some(line_hint) = line_hint {
        return items
            .iter()
            .find(|item| item.line_start <= line_hint && line_hint <= item.line_end);
    }

    items.first()
}

fn fallback_item_label(path: &str, fallback: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            let trimmed = fallback.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .unwrap_or_else(|| path.to_string())
}

fn first_signature_line(text: &str) -> &str {
    first_code_signature_line(text)
}

fn infer_generic_ast_node_kind(language_id: &CodeLanguageId, signature: &str) -> CodeAstNodeKind {
    if language_id.as_str() == "toml" {
        return if signature.trim_start().starts_with('[') {
            CodeAstNodeKind::Module
        } else {
            CodeAstNodeKind::Constant
        };
    }

    let normalized_signature = signature.trim_start();
    if normalized_signature.starts_with("fn ")
        || normalized_signature.starts_with("pub fn ")
        || normalized_signature.starts_with("function ")
        || normalized_signature.starts_with("def ")
        || normalized_signature.starts_with("async def ")
        || normalized_signature.starts_with("fun ")
        || normalized_signature.starts_with("func ")
    {
        return CodeAstNodeKind::Function;
    }
    if normalized_signature.starts_with("struct ")
        || normalized_signature.starts_with("pub struct ")
        || normalized_signature.starts_with("class ")
        || normalized_signature.starts_with("data class ")
        || normalized_signature.starts_with("interface ")
        || normalized_signature.starts_with("impl ")
    {
        return CodeAstNodeKind::Type;
    }
    if normalized_signature.starts_with("const ")
        || normalized_signature.starts_with("pub const ")
        || normalized_signature.starts_with("let ")
    {
        return CodeAstNodeKind::Constant;
    }

    CodeAstNodeKind::Other
}

fn generic_ast_semantic_type(kind: CodeAstNodeKind) -> &'static str {
    match kind {
        CodeAstNodeKind::Module => "module",
        CodeAstNodeKind::Function => "function",
        CodeAstNodeKind::Type => "type",
        CodeAstNodeKind::Constant => "constant",
        CodeAstNodeKind::ExternalSymbol => "externalSymbol",
        CodeAstNodeKind::Other => "other",
    }
}

fn supports_generic_code_blocks(language_id: &CodeLanguageId) -> bool {
    language_id.as_str() != "toml"
}
