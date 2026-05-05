use super::{RetrievalChunkLineExt, build_code_ast_retrieval_atom};
use crate::studio::types::{CodeAstRetrievalAtom, CodeAstRetrievalAtomScope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeBlockKind {
    Validation,
    Execution,
    Return,
}

#[derive(Debug, Clone)]
struct RawBlockSegment {
    start: usize,
    end: usize,
    lines: Vec<String>,
}

fn resolve_body_start_index(content_lines: &[&str], declaration_line: Option<usize>) -> usize {
    let Some(declaration_line) = declaration_line else {
        return 0;
    };
    if declaration_line == 0 {
        return 0;
    }

    let declaration_index = declaration_line.saturating_sub(1);
    for (index, raw_line) in content_lines.iter().enumerate().skip(declaration_index) {
        let current = raw_line.trim();
        if current.is_empty() {
            continue;
        }

        let has_body_delimiter = current.contains('{')
            || current.contains("=>")
            || current.starts_with("begin")
            || current.starts_with("algorithm")
            || current.starts_with("equation");

        if index == declaration_index {
            if has_body_delimiter {
                return index + 1;
            }
            continue;
        }

        if has_body_delimiter {
            return index + 1;
        }
    }

    declaration_line
}

fn collect_segments(
    content_lines: &[&str],
    declaration_line: Option<usize>,
) -> Vec<RawBlockSegment> {
    let body_start_index = resolve_body_start_index(content_lines, declaration_line);
    let mut segments = Vec::new();
    let mut current: Option<RawBlockSegment> = None;

    for (offset, raw_line) in content_lines.iter().enumerate().skip(body_start_index) {
        let absolute_line = offset + 1;
        if raw_line.trim().is_empty() {
            if let Some(segment) = current.take()
                && !segment.lines.is_empty()
            {
                segments.push(segment);
            }
            continue;
        }

        let segment = current.get_or_insert_with(|| RawBlockSegment {
            start: absolute_line,
            end: absolute_line,
            lines: Vec::new(),
        });
        segment.lines.push((*raw_line).to_string());
        segment.end = absolute_line;
    }

    if let Some(segment) = current.take()
        && !segment.lines.is_empty()
    {
        segments.push(segment);
    }

    segments
}

fn classify_block_kind(lines: &[String]) -> CodeBlockKind {
    let text = lines.join("\n");
    let lower = text.to_ascii_lowercase();

    let validation_like = text.lines().any(|line| {
        matches!(line.trim_start(), line if line.starts_with("if ")
            || line.starts_with("if(")
            || line.starts_with("guard ")
            || line.starts_with("assert")
            || line.starts_with("ensure")
            || line.starts_with("require")
            || line.starts_with("check"))
    });

    if validation_like
        || lower.contains("return err")
        || lower.contains("panic!")
        || lower.contains("throw")
        || lower.contains("raise")
    {
        return CodeBlockKind::Validation;
    }

    let return_like = text
        .lines()
        .any(|line| line.trim_start().starts_with("return "))
        || lower.contains("ok(")
        || lower.contains("err(")
        || lower.contains("some(")
        || lower.contains("none(");
    if return_like {
        return CodeBlockKind::Return;
    }

    CodeBlockKind::Execution
}

fn block_semantic_type(kind: CodeBlockKind) -> &'static str {
    match kind {
        CodeBlockKind::Validation => "validation",
        CodeBlockKind::Execution => "execution",
        CodeBlockKind::Return => "return",
    }
}

fn build_block_title(kind: CodeBlockKind, lines: &[String]) -> String {
    let head = lines
        .iter()
        .find_map(|line| {
            let trimmed = line.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or_default();

    match kind {
        CodeBlockKind::Validation => {
            if head.is_empty() {
                "Validation Block".to_string()
            } else {
                format!("Validation Block · {head}")
            }
        }
        CodeBlockKind::Execution => {
            if head.is_empty() {
                "Execution Block".to_string()
            } else {
                format!("Execution Block · {head}")
            }
        }
        CodeBlockKind::Return => {
            if head.is_empty() {
                "Return Path".to_string()
            } else {
                format!("Return Path · {head}")
            }
        }
    }
}

pub(crate) fn build_code_block_retrieval_atoms(
    path: &str,
    declaration_line: Option<usize>,
    source_content: &str,
) -> Vec<CodeAstRetrievalAtom> {
    let content_lines = source_content.lines().collect::<Vec<_>>();
    if content_lines.is_empty() {
        return Vec::new();
    }

    let segments = collect_segments(&content_lines, declaration_line);
    if segments.is_empty() {
        return Vec::new();
    }

    let mut grouped: Vec<(CodeBlockKind, Vec<RawBlockSegment>)> = Vec::new();
    for kind in [
        CodeBlockKind::Validation,
        CodeBlockKind::Execution,
        CodeBlockKind::Return,
    ] {
        let bucket = segments
            .iter()
            .filter(|segment| classify_block_kind(&segment.lines) == kind)
            .cloned()
            .collect::<Vec<_>>();
        if !bucket.is_empty() {
            grouped.push((kind, bucket));
        }
    }

    grouped
        .into_iter()
        .map(|(kind, segments)| {
            let start = segments
                .iter()
                .map(|segment| segment.start)
                .min()
                .unwrap_or(1);
            let end = segments
                .iter()
                .map(|segment| segment.end)
                .max()
                .unwrap_or(start);
            let excerpt = segments
                .iter()
                .flat_map(|segment| {
                    let mut lines = segment.lines.iter().take(6).cloned().collect::<Vec<_>>();
                    if segment.lines.len() > 6 {
                        lines.push("…".to_string());
                    }
                    lines
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut atom = build_code_ast_retrieval_atom(
                format!("block:{}:{}-{}", block_semantic_type(kind), start, end).as_str(),
                path,
                CodeAstRetrievalAtomScope::Block,
                block_semantic_type(kind),
                format!("l{start}-l{end}").as_str(),
                excerpt.as_str(),
            )
            .with_lines(start, end);
            atom.display_label = Some(build_block_title(
                kind,
                segments
                    .first()
                    .map_or(&[], |segment| segment.lines.as_slice()),
            ));
            atom.excerpt = Some(excerpt);
            atom
        })
        .collect()
}
