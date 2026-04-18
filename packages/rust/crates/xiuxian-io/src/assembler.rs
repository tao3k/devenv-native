//! Context Assembler - Parallel I/O + Templating + Token Counting
//!
//! This module provides the core context hydration logic for skills.
//! It combines parallel file reading, template rendering, and token counting
//! into a single efficient operation.

use std::borrow::Borrow;
use std::path::{Path, PathBuf};

use minijinja::Environment;
use rayon::prelude::*;
use serde_json::Value;

use crate::error::{IoError, Result};
use xiuxian_tokenizer::count_tokens;

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
/// Combines parallel I/O (rayon), template rendering (minijinja),
/// and token counting (xiuxian-tokenizer) for efficient context hydration.
#[derive(Debug, Clone)]
pub struct ContextAssembler {
    env: Environment<'static>,
}

impl ContextAssembler {
    /// Create a new context assembler with default settings.
    #[must_use]
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        Self { env }
    }

    /// Assemble skill context from main file and references.
    ///
    /// This method:
    /// 1. Reads the main skill file and all references in parallel
    /// 2. Renders the main template with the provided variables
    /// 3. Assembles the final content with proper formatting
    /// 4. Counts tokens using xiuxian-tokenizer
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
        &self,
        main_path: impl AsRef<Path>,
        ref_paths: impl AsRef<[PathBuf]>,
        variables: impl Borrow<Value>,
    ) -> Result<AssemblyResult> {
        let main_path = main_path.as_ref();
        let ref_paths = ref_paths.as_ref();
        let variables = variables.borrow();

        // 1. [Parallel I/O] Read main file and references concurrently
        let (main_res, refs_res) = rayon::join(
            || std::fs::read_to_string(main_path),
            || {
                ref_paths
                    .par_iter()
                    .map(|p| (p.clone(), std::fs::read_to_string(p)))
                    .collect::<Vec<_>>()
            },
        );

        let main_template = main_res.map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                IoError::NotFound(main_path.display().to_string())
            } else {
                IoError::System(error)
            }
        })?;

        // 2. [Templating] Render the main template
        let rendered_main = self
            .env
            .render_str(&main_template, variables)
            .unwrap_or_else(|e| format!("[Template Error: {e}]"));

        // 3. [Assembly] Build the final buffer
        let mut buffer = String::with_capacity(rendered_main.len() + 2048);
        buffer.push_str("# Active Protocol\n");
        buffer.push_str(&rendered_main);

        let mut missing = Vec::new();

        if !ref_paths.is_empty() {
            buffer.push_str("\n\n# Required References\n");
            for (path, content_res) in refs_res {
                match content_res {
                    Ok(c) => {
                        buffer.push_str("\n## ");
                        if let Some(name) = path.file_name() {
                            buffer.push_str(&name.to_string_lossy());
                        }
                        buffer.push('\n');
                        buffer.push_str(&c);
                    }
                    Err(_) => missing.push(path),
                }
            }
        }

        // 4. [Token Counting] using xiuxian-tokenizer
        let count = count_tokens(&buffer);

        Ok(AssemblyResult {
            content: buffer,
            token_count: count,
            missing_refs: missing,
        })
    }
}

impl Default for ContextAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "assembler"))]
#[path = "../tests/unit/assembler/mod.rs"]
mod tests;
