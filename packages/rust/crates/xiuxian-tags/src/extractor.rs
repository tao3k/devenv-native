//! High-performance AST-based symbol extractor for code navigation.
//!
//! Extracts symbols (functions, classes, etc.) from source code using ast-grep
//! patterns. Part of The Cartographer.

use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use xiuxian_ast::{
    AstLanguage, Doc, LanguageExt, MatcherExt, MetaVariable, NodeMatch, Pattern, SupportLang,
};

use crate::error::{SearchError, TagError};
use crate::patterns::{
    JS_CLASS_PATTERN, JS_FN_PATTERN, PYTHON_ASYNC_DEF_PATTERN, PYTHON_CLASS_PATTERN,
    PYTHON_DEF_PATTERN, RUST_ENUM_PATTERN, RUST_FN_PATTERN, RUST_IMPL_PATTERN, RUST_STRUCT_PATTERN,
    RUST_TRAIT_PATTERN, TS_INTERFACE_PATTERN,
};
use crate::types::{SearchConfig, SearchMatch, Symbol, SymbolKind};

fn append_fmt(buffer: &mut String, args: fmt::Arguments<'_>) {
    buffer
        .write_fmt(args)
        .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
}

/// High-performance AST-based symbol extractor for code navigation.
///
/// Extracts symbols (functions, classes, etc.) from source code using ast-grep
/// patterns. Part of The Cartographer.
pub struct TagExtractor;

struct SearchInput {
    path: PathBuf,
    lang: SupportLang,
}

impl TagExtractor {
    /// Generate a symbolic outline for a file
    /// Returns formatted string ready for LLM consumption.
    ///
    /// # Errors
    ///
    /// Returns `TagError` when the file cannot be read safely.
    pub fn outline_file<P: AsRef<Path>>(
        path: P,
        language: Option<&str>,
    ) -> Result<String, TagError> {
        let path = path.as_ref();
        let content = xiuxian_io::read_text_safe(path, 1024 * 1024)?; // 1MB limit for outlining

        let lang = match language {
            Some(l) => match SupportLang::from_str(l) {
                Ok(lang) => lang,
                Err(_) => return Ok(format!("[No outline available for {l}")),
            },
            None => {
                if let Some(lang) = SupportLang::from_path(path) {
                    lang
                } else {
                    return Ok(format!("[No outline available for {}]", path.display()));
                }
            }
        };

        let symbols = match lang {
            SupportLang::Python => Self::extract_python(&content),
            SupportLang::Rust => Self::extract_rust(&content),
            SupportLang::JavaScript => Self::extract_js(&content),
            SupportLang::TypeScript => Self::extract_ts(&content),
            _ => return Ok(format!("[No outline available for {lang:?}]")),
        };

        if symbols.is_empty() {
            return Ok(format!("[No symbols found in {}]", path.display()));
        }

        // Build CCA-style outline
        let mut output = String::new();
        append_fmt(
            &mut output,
            format_args!("// OUTLINE: {}\n", path.display()),
        );
        append_fmt(
            &mut output,
            format_args!("// Total symbols: {}\n", symbols.len()),
        );

        for sym in &symbols {
            let kind_str = format!("{:?}", sym.kind).to_lowercase();
            let kind_label = format!("[{kind_str}]");
            append_fmt(
                &mut output,
                format_args!(
                    "L{: <4} {: <12} {} {}\n",
                    sym.line, kind_label, sym.name, sym.signature
                ),
            );
        }

        Ok(output)
    }

    // ============================================================================
    // The Hunter - Structural Code Search
    // ============================================================================

    /// Search for a pattern in a single file using ast-grep
    ///
    /// # Arguments
    /// * `path` - Path to the file to search
    /// * `pattern` - ast-grep pattern (e.g., "connect($ARGS)", "class $NAME")
    /// * `language` - Optional language hint (python, rust, javascript, typescript)
    ///
    /// # Returns
    /// Formatted string showing all matches with context.
    ///
    /// # Errors
    ///
    /// Returns `SearchError` when the file cannot be read, the language cannot
    /// be resolved, or the pattern is invalid for the selected language.
    pub fn search_file<P: AsRef<Path>>(
        path: P,
        pattern: &str,
        language: Option<&str>,
    ) -> Result<String, SearchError> {
        let path = path.as_ref();
        let content = xiuxian_io::read_text_safe(path, 1024 * 1024)?;

        let lang = match language {
            Some(l) => match SupportLang::from_str(l) {
                Ok(lang) => lang,
                Err(_) => return Err(SearchError::UnsupportedLanguage(l.to_string())),
            },
            None => {
                if let Some(lang) = SupportLang::from_path(path) {
                    lang
                } else {
                    let ext = path.extension().map_or_else(
                        || "unknown".to_string(),
                        |extension| extension.to_string_lossy().to_string(),
                    );
                    return Err(SearchError::UnsupportedLanguage(ext));
                }
            }
        };

        let matches = Self::search_content(&content, pattern, lang, path)?;

        if matches.is_empty() {
            return Ok(format!(
                "[No matches for pattern '{}' in {}]",
                pattern,
                path.display()
            ));
        }

        // Build formatted output
        let mut output = String::new();
        append_fmt(&mut output, format_args!("// SEARCH: {}\n", path.display()));
        append_fmt(&mut output, format_args!("// Pattern: {pattern}\n"));
        append_fmt(
            &mut output,
            format_args!("// Total matches: {}\n", matches.len()),
        );

        for m in &matches {
            append_fmt(
                &mut output,
                format_args!("L{: <4}:{: <3} {}\n", m.line, m.column, m.content),
            );
        }

        Ok(output)
    }

    /// Search for a pattern in a directory recursively
    ///
    /// # Arguments
    /// * `dir` - Directory to search in
    /// * `pattern` - ast-grep pattern
    /// * `config` - Search configuration
    ///
    /// # Returns
    /// Formatted string showing all matches across files.
    ///
    /// # Errors
    ///
    /// Returns `SearchError` when a readable file contains an invalid pattern
    /// match context for the selected language.
    pub fn search_directory<P: AsRef<Path>>(
        dir: P,
        pattern: &str,
        config: SearchConfig,
    ) -> Result<String, SearchError> {
        let dir = dir.as_ref();
        let SearchConfig {
            file_pattern: _file_pattern,
            max_file_size,
            max_matches_per_file,
            languages: _languages,
        } = config;
        let search_inputs = Self::directory_search_inputs(dir, max_file_size);
        let file_count = search_inputs.len();
        let all_matches =
            Self::collect_directory_matches(search_inputs, pattern, max_matches_per_file * 10)?;

        if all_matches.is_empty() {
            return Ok(format!(
                "[No matches for pattern '{}' in {}]",
                pattern,
                dir.display()
            ));
        }

        // Group matches by file
        let mut output = String::new();
        append_fmt(&mut output, format_args!("// SEARCH: {}\n", dir.display()));
        append_fmt(&mut output, format_args!("// Pattern: {pattern}\n"));
        append_fmt(
            &mut output,
            format_args!("// Files searched: {file_count}\n"),
        );
        append_fmt(
            &mut output,
            format_args!("// Total matches: {}\n", all_matches.len()),
        );

        // Group by file
        let mut current_file = String::new();
        for m in all_matches {
            if m.path != current_file {
                current_file.clone_from(&m.path);
                append_fmt(&mut output, format_args!("\n// File: {current_file}\n"));
            }
            append_fmt(
                &mut output,
                format_args!("L{: <4}:{: <3} {}\n", m.line, m.column, m.content),
            );
        }

        Ok(output)
    }

    /// Internal: Search content for a pattern
    fn search_content(
        content: &str,
        pattern_str: &str,
        lang: SupportLang,
        path: &Path,
    ) -> Result<Vec<SearchMatch>, SearchError> {
        let root = lang.ast_grep(content);
        let root_node = root.root();

        // Create the pattern using try_new to handle errors gracefully
        let pattern = match Pattern::try_new(pattern_str, lang) {
            Ok(p) => p,
            Err(e) => return Err(SearchError::Pattern(e.to_string())),
        };

        Ok(root_node
            .dfs()
            .filter_map(|node| pattern.match_node(node.clone()))
            .take(100)
            .map(|matched| Self::search_match_from_node(path, &matched))
            .collect())
    }

    fn directory_search_inputs(dir: &Path, max_file_size: u64) -> Vec<SearchInput> {
        use walkdir::WalkDir;

        WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .metadata()
                    .map_or(true, |metadata| metadata.len() <= max_file_size)
            })
            .filter_map(|entry| {
                let lang = Self::language_from_path(entry.path())?;
                Some(SearchInput {
                    path: entry.path().to_path_buf(),
                    lang,
                })
            })
            .collect()
    }

    fn collect_directory_matches(
        search_inputs: Vec<SearchInput>,
        pattern: &str,
        max_matches: usize,
    ) -> Result<Vec<SearchMatch>, SearchError> {
        search_inputs
            .into_iter()
            .filter_map(|input| {
                let content = std::fs::read_to_string(&input.path).ok()?;
                Some((input, content))
            })
            .try_fold(Vec::new(), |mut all_matches, (input, content)| {
                if all_matches.len() >= max_matches {
                    return Ok(all_matches);
                }
                all_matches.extend(Self::search_content(
                    &content,
                    pattern,
                    input.lang,
                    &input.path,
                )?);
                Ok(all_matches)
            })
    }

    fn language_from_path(path: &Path) -> Option<SupportLang> {
        match path.extension().and_then(|extension| extension.to_str())? {
            "py" => Some(SupportLang::Python),
            "rs" => Some(SupportLang::Rust),
            "js" => Some(SupportLang::JavaScript),
            "ts" => Some(SupportLang::TypeScript),
            _ => None,
        }
    }

    fn search_match_from_node<D: Doc>(path: &Path, matched: &NodeMatch<D>) -> SearchMatch {
        let env = matched.get_env();
        let captures = env
            .get_matched_variables()
            .filter_map(|meta_variable| match meta_variable {
                MetaVariable::Capture(name, _) | MetaVariable::MultiCapture(name) => env
                    .get_match(&name)
                    .map(|captured| (name, captured.text().to_string())),
                MetaVariable::Dropped(_) | MetaVariable::Multiple => None,
            })
            .collect();

        SearchMatch {
            path: path.to_string_lossy().to_string(),
            line: matched.start_pos().line(),
            column: 0,
            content: matched.text().to_string(),
            captures,
        }
    }

    /// Extract symbols from Python source using AST patterns
    fn extract_python(content: &str) -> Vec<Symbol> {
        let lang = SupportLang::Python;
        let root = lang.ast_grep(content);
        let root_node = root.root();

        let mut symbols = Vec::new();

        // Extract classes
        let pattern = Pattern::new(PYTHON_CLASS_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("class {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract functions
        let pattern = Pattern::new(PYTHON_DEF_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("def {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract async functions
        let pattern = Pattern::new(PYTHON_ASYNC_DEF_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("async def {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::AsyncFunction,
                    line,
                    signature: sig,
                });
            }
        }

        // Sort by line number and deduplicate
        symbols.sort_by_key(|s| s.line);
        symbols
    }

    /// Extract symbols from Rust source using AST patterns
    fn extract_rust(content: &str) -> Vec<Symbol> {
        let lang = SupportLang::Rust;
        let root = lang.ast_grep(content);
        let root_node = root.root();

        let mut symbols = Vec::new();

        // Extract structs
        let pattern = Pattern::new(RUST_STRUCT_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("struct {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Struct,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract functions
        let pattern = Pattern::new(RUST_FN_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("fn {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract enums
        let pattern = Pattern::new(RUST_ENUM_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("enum {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Enum,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract traits
        let pattern = Pattern::new(RUST_TRAIT_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("trait {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Trait,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract impl blocks
        let pattern = Pattern::new(RUST_IMPL_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("impl {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Impl,
                    line,
                    signature: sig,
                });
            }
        }

        symbols.sort_by_key(|s| s.line);
        symbols
    }

    /// Extract symbols from JavaScript source
    fn extract_js(content: &str) -> Vec<Symbol> {
        let lang = SupportLang::JavaScript;
        let root = lang.ast_grep(content);
        let root_node = root.root();

        let mut symbols = Vec::new();

        // Extract classes
        let pattern = Pattern::new(JS_CLASS_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("class {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    line,
                    signature: sig,
                });
            }
        }

        // Extract functions
        let pattern = Pattern::new(JS_FN_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("function {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function,
                    line,
                    signature: sig,
                });
            }
        }

        symbols.sort_by_key(|s| s.line);
        symbols
    }

    /// Extract symbols from TypeScript source
    fn extract_ts(content: &str) -> Vec<Symbol> {
        let lang = SupportLang::TypeScript;
        let root = lang.ast_grep(content);
        let root_node = root.root();

        let mut symbols = Vec::new();

        // First extract JS-like symbols
        symbols.extend(Self::extract_js(content));

        // Extract interfaces
        let pattern = Pattern::new(TS_INTERFACE_PATTERN, lang);
        for node in root_node.dfs() {
            if let Some(m) = pattern.match_node(node.clone()) {
                let name = Self::get_capture(&m, "NAME");
                let line = m.start_pos().line();
                let sig = format!("interface {name}");
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Interface,
                    line,
                    signature: sig,
                });
            }
        }

        symbols.sort_by_key(|s| s.line);
        symbols
    }

    /// Get the text of a variable capture from a matched node
    fn get_capture<D: Doc>(m: &NodeMatch<D>, capture: &str) -> String {
        m.get_env()
            .get_match(capture)
            .map(|n| n.text().to_string())
            .unwrap_or_default()
    }
}
