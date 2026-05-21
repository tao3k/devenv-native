//! `zhenfa_router::native::sentinel::analysis` owns Wendao native sentinel analysis behavior.

use std::path::Path;
use std::sync::OnceLock;

use log::info;

use crate::LinkGraphIndex;
use crate::link_graph::{PageIndexNode, SymbolRef};
use crate::parsers::markdown::code_observation::path_matches_scope;

use super::types::{AffectedDoc, DriftConfidence, SemanticDriftSignal};

static RE_FN: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_STRUCT: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_CLASS: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_ENUM: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_METHOD: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_TRAIT: OnceLock<Option<regex::Regex>> = OnceLock::new();
static RE_IMPL: OnceLock<Option<regex::Regex>> = OnceLock::new();

struct SymbolExtractor {
    regex: &'static OnceLock<Option<regex::Regex>>,
    source: &'static str,
    unique: bool,
}

static SYMBOL_EXTRACTORS: [SymbolExtractor; 7] = [
    SymbolExtractor {
        regex: &RE_FN,
        source: r"\bfn\s+([a-z_][a-z0-9_]*)",
        unique: false,
    },
    SymbolExtractor {
        regex: &RE_STRUCT,
        source: r"\bstruct\s+([A-Z][a-zA-Z0-9_]*)",
        unique: false,
    },
    SymbolExtractor {
        regex: &RE_CLASS,
        source: r"\bclass\s+([A-Z][a-zA-Z0-9_]*)",
        unique: false,
    },
    SymbolExtractor {
        regex: &RE_ENUM,
        source: r"\benum\s+([A-Z][a-zA-Z0-9_]*)",
        unique: false,
    },
    SymbolExtractor {
        regex: &RE_METHOD,
        source: r"\b(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(",
        unique: true,
    },
    SymbolExtractor {
        regex: &RE_TRAIT,
        source: r"\btrait\s+([A-Z][a-zA-Z0-9_]*)",
        unique: false,
    },
    SymbolExtractor {
        regex: &RE_IMPL,
        source: r"\bimpl\s+(?:[A-Z][a-zA-Z0-9_]*\s+for\s+)?([A-Z][a-zA-Z0-9_]*)",
        unique: false,
    },
];

struct SourceChangeContext {
    path: String,
    stem: String,
    stem_lower: String,
}

fn capture_symbol(pattern: &str, regex: Option<&regex::Regex>) -> Option<String> {
    regex.and_then(|compiled| {
        compiled
            .captures(pattern)
            .and_then(|caps| caps.get(1))
            .map(|capture| capture.as_str().to_string())
    })
}

fn push_captured_symbol(symbols: &mut Vec<String>, pattern: &str, regex: Option<&regex::Regex>) {
    if let Some(symbol) = capture_symbol(pattern, regex) {
        symbols.push(symbol);
    }
}

fn push_unique_captured_symbol(
    symbols: &mut Vec<String>,
    pattern: &str,
    regex: Option<&regex::Regex>,
) {
    if let Some(symbol) = capture_symbol(pattern, regex)
        && !symbols.contains(&symbol)
    {
        symbols.push(symbol);
    }
}

fn compiled_symbol_regex(extractor: &SymbolExtractor) -> Option<&regex::Regex> {
    extractor
        .regex
        .get_or_init(|| regex::Regex::new(extractor.source).ok())
        .as_ref()
}

fn push_extracted_symbol(symbols: &mut Vec<String>, pattern: &str, extractor: &SymbolExtractor) {
    let regex = compiled_symbol_regex(extractor);
    if extractor.unique {
        push_unique_captured_symbol(symbols, pattern, regex);
    } else {
        push_captured_symbol(symbols, pattern, regex);
    }
}

/// Extract core symbols from an observation pattern.
///
/// This is a heuristic extraction for the Symbol-to-Node Inverted Index.
/// Patterns like `fn process_data($$$)` yield `["process_data"]`.
/// Patterns like `struct User { $$$ }` yield `["User"]`.
#[must_use]
pub fn extract_pattern_symbols(pattern: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for extractor in &SYMBOL_EXTRACTORS {
        push_extracted_symbol(&mut symbols, pattern, extractor);
    }
    symbols
}

/// Compute Blake3 hash of a file's content.
#[must_use]
pub fn compute_file_hash(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(blake3::hash(content.as_bytes()).to_hex().to_string())
}

/// Check if a file path matches the observation's scope filter.
///
/// Returns `true` if:
/// - The scope is `None` (no filtering)
/// - The scope matches the file path using glob pattern matching
///
/// Returns `false` if:
/// - The scope is `Some` but doesn't match the file path
#[must_use]
pub(crate) fn matches_scope_filter(file_path: &str, scope: Option<&str>) -> bool {
    match scope {
        None => true, // No scope means match all files
        Some(scope_pattern) => path_matches_scope(file_path, scope_pattern),
    }
}

fn add_symbol_refs_to_signal(
    signal: &mut SemanticDriftSignal,
    symbol_refs: &[SymbolRef],
    file_path: &str,
) {
    for sym_ref in symbol_refs {
        if !matches_scope_filter(file_path, sym_ref.scope.as_deref()) {
            continue;
        }

        if signal
            .affected_docs
            .iter()
            .any(|doc| doc.node_id == sym_ref.node_id)
        {
            continue;
        }

        let affected = AffectedDoc::new(
            &sym_ref.doc_id,
            &sym_ref.pattern,
            &sym_ref.language,
            &sym_ref.node_id,
        )
        .with_line(sym_ref.line_number.unwrap_or(0));

        signal.add_affected_doc(affected);
    }
}

fn has_explicit_reference(affected_docs: &[AffectedDoc], file_stem: &str) -> bool {
    let function_pattern = format!("fn {file_stem}");
    let struct_pattern = format!("struct {file_stem}");
    let class_pattern = format!("class {file_stem}");

    affected_docs.iter().any(|doc| {
        doc.matching_pattern.contains(&function_pattern)
            || doc.matching_pattern.contains(&struct_pattern)
            || doc.matching_pattern.contains(&class_pattern)
    })
}

fn source_change_context(path: &Path) -> SourceChangeContext {
    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_string();
    SourceChangeContext {
        path: path.to_string_lossy().to_string(),
        stem_lower: file_stem.to_lowercase(),
        stem: file_stem,
    }
}

fn add_index_symbol_matches(
    index: &LinkGraphIndex,
    context: &SourceChangeContext,
    signal: &mut SemanticDriftSignal,
) {
    if !index.has_symbols() {
        return;
    }

    for symbol in source_symbol_variants(context.stem.as_str()) {
        add_symbol_variant_matches(index, symbol.as_str(), &context.path, signal);
    }
}

fn source_symbol_variants(file_stem: &str) -> Vec<String> {
    let mut variants = vec![file_stem.to_string()];
    let snake_variant = file_stem.to_lowercase().replace('-', "_");
    if snake_variant != file_stem {
        variants.push(snake_variant);
    }
    variants.push(to_pascal_case(file_stem));
    variants
}

fn add_symbol_variant_matches(
    index: &LinkGraphIndex,
    symbol: &str,
    file_path: &str,
    signal: &mut SemanticDriftSignal,
) {
    let Some(symbol_refs) = index.lookup_symbol(symbol) else {
        return;
    };
    info!(
        "Phase 6.4: O(1) cache hit for symbol '{}' ({} refs)",
        symbol,
        symbol_refs.len()
    );
    add_symbol_refs_to_signal(signal, symbol_refs, file_path);
}

fn add_heuristic_matches(
    index: &LinkGraphIndex,
    context: &SourceChangeContext,
    signal: &mut SemanticDriftSignal,
) {
    if !signal.affected_docs.is_empty() {
        return;
    }

    info!("Phase 6: Cache miss, falling back to heuristic traversal");
    for (doc_id, nodes) in index.all_page_index_trees() {
        traverse_nodes_for_observations(nodes, doc_id, &context.stem, &context.stem_lower, signal);
    }
}

fn finalize_source_change_signal(
    mut signal: SemanticDriftSignal,
    context: &SourceChangeContext,
) -> Vec<SemanticDriftSignal> {
    if signal.affected_docs.is_empty() {
        return Vec::new();
    }

    signal.update_confidence(source_change_confidence(&signal, context.stem.as_str()));

    info!(
        "Phase 6: {} documents potentially affected by source change.",
        signal.affected_docs.len()
    );

    vec![signal]
}

fn source_change_confidence(signal: &SemanticDriftSignal, file_stem: &str) -> DriftConfidence {
    if has_explicit_reference(&signal.affected_docs, file_stem) {
        DriftConfidence::High
    } else if signal.affected_docs.len() <= 3 {
        DriftConfidence::Medium
    } else {
        DriftConfidence::Low
    }
}

/// Phase 6: Core logic for propagating source changes to documentation.
///
/// Uses the Symbol-to-Node Inverted Index for O(1) lookup when available,
/// falling back to heuristic traversal when the index is empty or misses.
///
/// # Phase 7.6: Scope Filtering
///
/// Observations with a `scope` filter only match files within the specified
/// path pattern. This prevents false positives when the same symbol exists
/// in multiple packages.
///
/// # Returns
///
/// A vector of `SemanticDriftSignal` events for each affected observation.
#[must_use]
pub fn propagate_source_change(index: &LinkGraphIndex, path: &Path) -> Vec<SemanticDriftSignal> {
    info!("Propagating semantic change from code: {}", path.display());

    let context = source_change_context(path);
    let mut signal = SemanticDriftSignal::new(&context.path, &context.stem);
    add_index_symbol_matches(index, &context, &mut signal);
    add_heuristic_matches(index, &context, &mut signal);
    finalize_source_change_signal(signal, &context)
}

/// Convert `snake_case` to `PascalCase`.
#[must_use]
pub(crate) fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Recursively traverse page index nodes to find matching observations.
fn traverse_nodes_for_observations(
    nodes: &[PageIndexNode],
    doc_id: &str,
    file_stem: &str,
    file_stem_lower: &str,
    signal: &mut SemanticDriftSignal,
) {
    for node in nodes {
        // Check observations in this node's metadata
        for obs in &node.metadata.observations {
            let pattern_lower = obs.pattern.to_lowercase();

            // Heuristic matching: pattern contains file stem or related symbols
            let matches = pattern_lower.contains(file_stem_lower)
                || obs.pattern.contains(&format!("{file_stem}_{file_stem}"))
                || obs.pattern.contains(&format!("{file_stem}::"))
                || obs.pattern.contains(&format!("{file_stem}."));

            if matches {
                let affected = AffectedDoc::new(
                    doc_id,
                    obs.pattern.clone(),
                    obs.language.clone(),
                    node.node_id.clone(),
                )
                .with_line(obs.line_number.unwrap_or(node.metadata.line_range.0));

                signal.add_affected_doc(affected);
            }
        }

        // Recurse into children
        traverse_nodes_for_observations(&node.children, doc_id, file_stem, file_stem_lower, signal);
    }
}
