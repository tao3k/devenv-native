use std::collections::HashSet;
use std::path::Path;

use xiuxian_ast::{Lang, extract_items, get_skeleton_patterns};

use crate::gateway::studio::types::{
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
    lang: Lang,
) -> CodeAstAnalysisResponse {
    let items = extract_generic_ast_items(repo_id.as_str(), path.as_str(), source_content, lang);
    let focus_item = focus_generic_ast_item(items.as_slice(), line_hint);
    let focus_node_id = focus_item.map(|item| item.id.clone());
    let mut nodes = Vec::with_capacity(items.len());
    let mut retrieval_atoms = Vec::with_capacity(items.len() * 2);

    for item in &items {
        nodes.push(CodeAstNode {
            id: item.id.clone(),
            label: item.label.clone(),
            kind: item.kind,
            path: Some(path.clone()),
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
            ("language".to_string(), lang.as_str().to_string()),
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

    if supports_generic_code_blocks(lang)
        && let Some(focus_item) = focus_item
    {
        retrieval_atoms.extend(build_code_block_retrieval_atoms(
            path.as_str(),
            Some(focus_item.line_start),
            source_content,
        ));
    }

    CodeAstAnalysisResponse {
        repo_id,
        path,
        language: lang.as_str().to_string(),
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
        focus_node_id,
        diagnostics: Vec::new(),
    }
}

fn extract_generic_ast_items(
    repo_id: &str,
    path: &str,
    source_content: &str,
    lang: Lang,
) -> Vec<GenericAstItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for pattern in get_skeleton_patterns(lang) {
        for result in extract_items(source_content, pattern, lang, Some(vec!["NAME"])) {
            let signature = first_signature_line(result.text.as_str()).to_string();
            if signature.is_empty() {
                continue;
            }

            let label = result
                .captures
                .get("NAME")
                .cloned()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| fallback_item_label(path, signature.as_str()));
            let dedupe_key = format!("{path}:{}:{}:{label}", result.line_start, result.line_end);
            if !seen.insert(dedupe_key) {
                continue;
            }

            items.push(GenericAstItem {
                id: format!(
                    "repo:{repo_id}:generic_ast:{}:{}:{}:{}",
                    path, result.line_start, result.line_end, label
                ),
                label,
                kind: infer_generic_ast_node_kind(lang, pattern, signature.as_str()),
                signature,
                line_start: result.line_start,
                line_end: result.line_end,
            });
        }
    }

    if !items.is_empty() {
        return items;
    }

    let label = fallback_item_label(path, source_content);
    let signature = first_signature_line(source_content).to_string();
    items.push(GenericAstItem {
        id: format!("repo:{repo_id}:generic_ast:{path}:1:1:{label}"),
        label,
        kind: if lang == Lang::Toml {
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
    text.lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or_default()
}

fn infer_generic_ast_node_kind(lang: Lang, pattern: &str, signature: &str) -> CodeAstNodeKind {
    if lang == Lang::Toml {
        return if pattern.contains('[') {
            CodeAstNodeKind::Module
        } else {
            CodeAstNodeKind::Constant
        };
    }

    let normalized_signature = signature.trim_start();
    let normalized_pattern = pattern.trim_start();
    if normalized_signature.starts_with("fn ")
        || normalized_signature.starts_with("pub fn ")
        || normalized_pattern.starts_with("fn ")
        || normalized_pattern.starts_with("pub fn ")
        || normalized_pattern.starts_with("function ")
        || normalized_pattern.starts_with("def ")
        || normalized_pattern.starts_with("async def ")
        || normalized_pattern.starts_with("fun ")
        || normalized_pattern.starts_with("func ")
    {
        return CodeAstNodeKind::Function;
    }
    if normalized_signature.starts_with("struct ")
        || normalized_signature.starts_with("pub struct ")
        || normalized_signature.starts_with("class ")
        || normalized_signature.starts_with("data class ")
        || normalized_signature.starts_with("interface ")
        || normalized_signature.starts_with("impl ")
        || normalized_pattern.contains("struct")
        || normalized_pattern.contains("class")
        || normalized_pattern.contains("interface")
        || normalized_pattern.starts_with("impl ")
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

fn supports_generic_code_blocks(lang: Lang) -> bool {
    !matches!(lang, Lang::Toml)
}
