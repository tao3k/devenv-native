//! AST-based code chunking for semantic partitioning.
//!
//! Provides functions to split source code into semantic chunks based on
//! AST patterns, enabling high-quality knowledge base construction.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::lang::Lang;
use crate::python::extract_python_docstring;
use crate::re_exports::{
    Doc, LanguageExt, MatcherExt, MetaVariable, NodeMatch, Pattern, SupportLang,
};

struct ChunkPatternSpec {
    chunk_idx: usize,
    chunk_type: ChunkType,
    search_pattern: Pattern,
}

/// Code chunk for semantic partitioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeChunk {
    /// Chunk identifier
    pub id: String,
    /// Chunk type (function, class, etc.)
    pub chunk_type: ChunkType,
    /// Raw code content
    pub content: String,
    /// Byte offset start
    pub start: usize,
    /// Byte offset end
    pub end: usize,
    /// Line number start (1-indexed)
    pub line_start: usize,
    /// Line number end (1-indexed)
    pub line_end: usize,
    /// Captured metadata (function name, class name, etc.)
    pub metadata: HashMap<String, String>,
    /// Docstring/comment content
    pub docstring: Option<String>,
}

/// Public typed boundary for chunk kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChunkType(String);

impl ChunkType {
    /// Creates a chunk type from a stable label.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the chunk type label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq<&str> for ChunkType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Request for semantic code chunking.
#[derive(Debug, Clone, Copy)]
pub struct ChunkCodeRequest<'a> {
    /// Source code content.
    pub content: &'a str,
    /// Path to the file for ID generation.
    pub file_path: &'a str,
    /// Programming language.
    pub lang: Lang,
    /// AST patterns to match.
    pub patterns: &'a [&'a str],
    /// Minimum lines for a chunk to be included.
    pub min_lines: usize,
    /// Maximum lines for a chunk, or 0 for no split limit.
    pub max_lines: usize,
}

/// Chunk a file into semantic units based on AST patterns.
///
/// # Arguments
/// * `content` - Source code content
/// * `file_path` - Path to the file (for ID generation)
/// * `lang` - Programming language
/// * `patterns` - AST patterns to match (e.g., `["def $NAME", "class $NAME"]`)
/// * `min_lines` - Minimum lines for a chunk to be included
/// * `max_lines` - Maximum lines for a chunk (splits large chunks, 0 = no limit)
///
/// # Returns
/// Vector of `CodeChunk` objects.
///
/// # Errors
/// Returns an error when language or pattern parsing fails.
pub fn chunk_code(request: ChunkCodeRequest<'_>) -> Result<Vec<CodeChunk>> {
    let lang_str = request.lang.as_str();
    let support_lang: SupportLang = lang_str
        .parse()
        .with_context(|| format!("Failed to parse language: {lang_str}"))?;
    chunk_code_with_lang(request, support_lang)
}

fn chunk_code_with_lang(
    request: ChunkCodeRequest<'_>,
    support_lang: SupportLang,
) -> Result<Vec<CodeChunk>> {
    let grep_result = support_lang.ast_grep(request.content);
    let root_node = grep_result.root();

    let file_name = Path::new(request.file_path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let specs = chunk_pattern_specs(request.patterns, support_lang)?;
    let mut chunks = specs
        .iter()
        .flat_map(|spec| chunks_for_pattern(request.content, file_name, &root_node, spec))
        .filter(|chunk| chunk.line_end - chunk.line_start + 1 >= request.min_lines)
        .collect::<Vec<_>>();

    // Sort by line number
    chunks.sort_by_key(|a| a.line_start);

    // Handle max_lines by splitting large chunks
    if request.max_lines > 0 {
        chunks = split_large_chunks(chunks, request.max_lines);
    }

    Ok(chunks)
}

fn chunks_for_pattern<D: Doc>(
    content: &str,
    file_name: &str,
    root_node: &ast_grep_core::Node<'_, D>,
    spec: &ChunkPatternSpec,
) -> Vec<CodeChunk> {
    root_node
        .dfs()
        .filter_map(|node| spec.search_pattern.match_node(node.clone()))
        .map(|matched| {
            matched_chunk(
                content,
                file_name,
                &spec.chunk_type,
                spec.chunk_idx,
                &matched,
            )
        })
        .collect()
}

fn chunk_pattern_specs(
    patterns: &[&str],
    support_lang: SupportLang,
) -> Result<Vec<ChunkPatternSpec>> {
    patterns
        .iter()
        .enumerate()
        .map(|(chunk_idx, pattern)| {
            let search_pattern = Pattern::try_new(pattern, support_lang)
                .with_context(|| format!("Failed to parse pattern: {pattern}"))?;
            Ok(ChunkPatternSpec {
                chunk_idx,
                chunk_type: detect_chunk_type(pattern, chunk_idx),
                search_pattern,
            })
        })
        .collect()
}

fn matched_chunk(
    content: &str,
    file_name: &str,
    chunk_type: &ChunkType,
    chunk_idx: usize,
    matched: &NodeMatch<impl Doc>,
) -> CodeChunk {
    let range = matched.range();
    let start = range.start;
    let end = range.end;
    let line_start = content[..start].lines().count() + 1;
    let line_end = content[..end].lines().count();
    let metadata = extract_chunk_metadata(matched);
    let id = generate_chunk_id(file_name, chunk_type.as_str(), &metadata, chunk_idx);
    let content = matched.text().to_string();

    CodeChunk {
        id,
        chunk_type: chunk_type.clone(),
        docstring: python_docstring_for_chunk(&content),
        content,
        start,
        end,
        line_start,
        line_end,
        metadata,
    }
}

fn extract_chunk_metadata(matched: &NodeMatch<impl Doc>) -> HashMap<String, String> {
    let env = matched.get_env();
    env.get_matched_variables()
        .filter_map(|mv| {
            let MetaVariable::Capture(name, _) = mv else {
                return None;
            };
            env.get_match(&name)
                .map(|captured| (name.clone(), captured.text().to_string()))
        })
        .collect()
}

fn python_docstring_for_chunk(content: &str) -> Option<String> {
    let doc = extract_python_docstring(content);
    if doc.is_empty() { None } else { Some(doc) }
}

/// Detect chunk type from pattern string
fn detect_chunk_type(pattern: &str, idx: usize) -> ChunkType {
    if pattern.contains("def $NAME") || pattern.contains("function $NAME") {
        ChunkType::new("function")
    } else if pattern.contains("class $NAME") {
        ChunkType::new("class")
    } else if pattern.contains("interface $NAME") {
        ChunkType::new("interface")
    } else if pattern.contains("struct $NAME") {
        ChunkType::new("struct")
    } else if pattern.contains("const $NAME") || pattern.contains("let $NAME") {
        ChunkType::new("variable")
    } else if pattern.contains("fn $NAME") {
        ChunkType::new("function")
    } else {
        ChunkType::new(format!("chunk_{idx}"))
    }
}

/// Generate unique chunk ID
fn generate_chunk_id(
    file_name: &str,
    chunk_type: &str,
    metadata: &HashMap<String, String>,
    idx: usize,
) -> String {
    if let Some(name) = metadata.get("NAME") {
        format!("{file_name}_{chunk_type}_{name}")
    } else {
        format!("{file_name}_{chunk_type}_{idx}")
    }
}

/// Split large chunks into smaller parts
fn split_large_chunks(chunks: Vec<CodeChunk>, max_lines: usize) -> Vec<CodeChunk> {
    chunks
        .into_iter()
        .flat_map(|chunk| split_chunk_or_keep(chunk, max_lines))
        .collect()
}

fn split_chunk_or_keep(chunk: CodeChunk, max_lines: usize) -> Vec<CodeChunk> {
    if chunk.line_end - chunk.line_start + 1 > max_lines {
        split_chunk(&chunk, max_lines)
    } else {
        vec![chunk]
    }
}

/// Split a large chunk into smaller parts
fn split_chunk(chunk: &CodeChunk, max_lines: usize) -> Vec<CodeChunk> {
    let total_lines = chunk.line_end - chunk.line_start + 1;
    if total_lines <= max_lines {
        return vec![chunk.clone()];
    }

    let lines: Vec<&str> = chunk.content.lines().collect();
    let num_parts = total_lines.div_ceil(max_lines);
    (0..num_parts)
        .filter_map(|index| split_chunk_part(chunk, &lines, max_lines, total_lines, index))
        .collect()
}

fn split_chunk_part(
    chunk: &CodeChunk,
    lines: &[&str],
    max_lines: usize,
    total_lines: usize,
    index: usize,
) -> Option<CodeChunk> {
    let start_line = index * max_lines;
    let end_line = std::cmp::min((index + 1) * max_lines, total_lines);
    if start_line >= lines.len() {
        return None;
    }

    let part_content = lines[start_line..end_line].join("\n");
    let trimmed_content = part_content.trim_end();
    let trimmed_len = trimmed_content.len();
    let part_start = chunk.start + (part_content.len() - trimmed_len);
    let part_end = chunk.start + part_content.len();

    Some(CodeChunk {
        id: format!("{}_part_{}", chunk.id, index),
        chunk_type: chunk.chunk_type.clone(),
        content: trimmed_content.to_string(),
        start: part_start,
        end: part_end,
        line_start: chunk.line_start + start_line,
        line_end: chunk.line_start + end_line - 1,
        metadata: chunk.metadata.clone(),
        docstring: if index == 0 {
            chunk.docstring.clone()
        } else {
            None
        },
    })
}
