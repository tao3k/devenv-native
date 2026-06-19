//! Context Assembler - Parallel I/O + Templating + Token Estimation
//!
//! This module provides the core context hydration logic for skills.
//! It combines parallel file reading, template rendering, and token estimation
//! into a single efficient operation.

use std::borrow::Borrow;
use std::path::{Path, PathBuf};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use tera::{Context, Tera};

use crate::error::{IoError, Result};

#[cfg(feature = "assembler")]
type SkillReferenceRead = (PathBuf, std::io::Result<String>);

/// Result of assembling skill context.
#[derive(Debug, Clone)]
pub struct AssemblyResult {
    /// The assembled content string.
    pub content: String,
    /// Token count of the content.
    pub token_count: usize,
    /// List of reference paths that could not be read.
    pub missing_refs: Vec<PathBuf>,
}

/// Context assembler for skill protocols.
///
/// Combines parallel I/O (rayon), template rendering (tera),
/// and lightweight token estimation for efficient context hydration.
#[derive(Debug, Clone, Default)]
pub struct ContextAssembler;

impl ContextAssembler {
    /// Create a new context assembler with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Assemble skill context from main file and references.
    ///
    /// This method:
    /// 1. Reads the main skill file and all references in parallel
    /// 2. Renders the main template with the provided variables
    /// 3. Assembles the final content with proper formatting
    /// 4. Estimates tokens for context metadata
    ///
    /// # Arguments
    ///
    /// * `main_path` - Path to the main `SKILL.md` file
    /// * `ref_paths` - List of paths to reference files
    /// * `variables` - JSON object with template variables
    ///
    /// # Returns
    ///
    /// `Result<AssemblyResult>` containing the assembled content and metadata
    ///
    /// # Errors
    ///
    /// Returns [`IoError::NotFound`] when the main file path does not exist and
    /// [`IoError::System`] for other main-file I/O failures.
    #[cfg(feature = "assembler")]
    pub fn assemble_skill(
        main_path: impl AsRef<Path>,
        ref_paths: impl AsRef<[PathBuf]>,
        variables: impl Borrow<Value>,
    ) -> Result<AssemblyResult> {
        assemble_skill_impl(main_path.as_ref(), ref_paths.as_ref(), variables.borrow())
    }
}

#[cfg(feature = "assembler")]
fn assemble_skill_impl(
    main_path: &Path,
    ref_paths: &[PathBuf],
    variables: &Value,
) -> Result<AssemblyResult> {
    let (main_template, refs_res) = read_skill_inputs(main_path, ref_paths)?;
    let rendered_main = render_skill_template(&main_template, variables);
    let (content, missing_refs) = assemble_skill_buffer(&rendered_main, ref_paths, refs_res);
    let token_count = count_tokens(&content);

    Ok(AssemblyResult {
        content,
        token_count,
        missing_refs,
    })
}

#[cfg(feature = "assembler")]
fn count_tokens(content: &str) -> usize {
    content.split_whitespace().count()
}

#[cfg(feature = "assembler")]
fn read_skill_inputs(
    main_path: &Path,
    ref_paths: &[PathBuf],
) -> Result<(String, Vec<SkillReferenceRead>)> {
    let (main_res, refs_res) = rayon::join(
        || std::fs::read_to_string(main_path),
        || {
            ref_paths
                .par_iter()
                .map(|path| (path.clone(), std::fs::read_to_string(path)))
                .collect::<Vec<_>>()
        },
    );

    main_res
        .map(|main_template| (main_template, refs_res))
        .map_err(|error| main_read_error(main_path, error))
}

#[cfg(feature = "assembler")]
fn main_read_error(main_path: &Path, error: std::io::Error) -> IoError {
    if error.kind() == std::io::ErrorKind::NotFound {
        IoError::NotFound(main_path.display().to_string())
    } else {
        IoError::System(error)
    }
}

#[cfg(feature = "assembler")]
fn render_skill_template(main_template: &str, variables: &Value) -> String {
    Context::from_value(variables.clone())
        .map_err(|error| format!("[Template Error: {error}]"))
        .and_then(|context| {
            Tera::one_off(main_template, &context, false)
                .map_err(|error| format!("[Template Error: {error}]"))
        })
        .unwrap_or_else(|error| error)
}

#[cfg(feature = "assembler")]
fn assemble_skill_buffer(
    rendered_main: &str,
    ref_paths: &[PathBuf],
    refs_res: Vec<(PathBuf, std::io::Result<String>)>,
) -> (String, Vec<PathBuf>) {
    let mut buffer = String::with_capacity(rendered_main.len() + 2048);
    buffer.push_str("# Active Protocol\n");
    buffer.push_str(rendered_main);

    if ref_paths.is_empty() {
        return (buffer, Vec::new());
    }

    buffer.push_str("\n\n# Required References\n");
    append_reference_sections(&mut buffer, refs_res)
}

#[cfg(feature = "assembler")]
fn append_reference_sections(
    buffer: &mut String,
    refs_res: Vec<(PathBuf, std::io::Result<String>)>,
) -> (String, Vec<PathBuf>) {
    let missing = refs_res
        .into_iter()
        .filter_map(|(path, content_res)| match content_res {
            Ok(content) => {
                append_reference_section(buffer, &path, &content);
                None
            }
            Err(_) => Some(path),
        })
        .collect();
    (std::mem::take(buffer), missing)
}

#[cfg(feature = "assembler")]
fn append_reference_section(buffer: &mut String, path: &Path, content: &str) {
    buffer.push_str("\n## ");
    if let Some(name) = path.file_name() {
        buffer.push_str(&name.to_string_lossy());
    }
    buffer.push('\n');
    buffer.push_str(content);
}

#[cfg(all(test, feature = "assembler"))]
#[path = "../tests/unit/assembler/mod.rs"]
mod tests;
