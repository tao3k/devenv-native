//! Root-doc canonical owner-name checks for LLM-friendly Rust module layouts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use proc_macro2::{LineColumn, Span};
use syn::spanned::Spanned;
use syn::{AttrStyle, Expr, ExprLit, File, Item, ItemMod, ItemUse, Lit, Meta, UseTree, Visibility};

const MIN_CHILD_MODULES: usize = 3;

/// Result of evaluating whether the canonical owner is named directly in the root doc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RootDocOwnerNameCheck {
    /// The file is not a root-seam candidate or the rule does not apply.
    NotApplicable,
    /// The root doc names the canonical owner with the real child-module path.
    CanonicalOwnerName,
    /// The root doc hints the owner with alias wording instead of the child-module path.
    DriftedOwnerName(RootDocOwnerNameMetrics),
}

/// Metrics for one internal root doc with owner-name drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RootDocOwnerNameMetrics {
    /// Rendered parent module declaration visibility.
    pub(crate) parent_visibility: String,
    /// Canonical visible owner module name.
    pub(crate) visible_owner: String,
    /// The inline-code owner hint named in the root doc.
    pub(crate) hinted_owner: String,
    /// 1-based line number of the first root doc attribute.
    pub(crate) doc_line_number: usize,
}

/// Check whether an internal root doc names the canonical owner module directly.
#[must_use]
pub(crate) fn check_root_doc_owner_name(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> RootDocOwnerNameCheck {
    if !is_root_seam_candidate(path) {
        return RootDocOwnerNameCheck::NotApplicable;
    }

    let Some(module_name) = root_module_name(path) else {
        return RootDocOwnerNameCheck::NotApplicable;
    };

    let Some(parent_visibility) = resolve_parent_visibility(path, &module_name, file_texts) else {
        return RootDocOwnerNameCheck::NotApplicable;
    };
    if matches!(parent_visibility, ParentModuleVisibility::Public) {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }

    let Ok(file) = syn::parse_file(text) else {
        return RootDocOwnerNameCheck::NotApplicable;
    };

    let child_modules = collect_child_modules(&file);
    if child_modules.len() < MIN_CHILD_MODULES {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }

    let visible_owners = collect_visible_owner_modules(&file, &child_modules);
    if visible_owners.len() != 1 {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }

    let Some(visible_owner) = visible_owners.iter().next().cloned() else {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    };

    let Some(root_doc) = collect_root_doc(&file) else {
        return RootDocOwnerNameCheck::NotApplicable;
    };

    let named_modules = child_module_positions(&root_doc.text, &child_modules);
    if named_modules.contains_key(&visible_owner) {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }
    if !named_modules.is_empty() {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }

    let Some(hinted_owner) = first_inline_owner_hint(&root_doc.text) else {
        return RootDocOwnerNameCheck::NotApplicable;
    };
    if inline_owner_hint_matches_module(&hinted_owner, &visible_owner) {
        return RootDocOwnerNameCheck::CanonicalOwnerName;
    }

    RootDocOwnerNameCheck::DriftedOwnerName(RootDocOwnerNameMetrics {
        parent_visibility: parent_visibility.rendered_declaration(&module_name),
        visible_owner,
        hinted_owner,
        doc_line_number: root_doc.line_number,
    })
}

fn is_root_seam_candidate(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(file_name, "lib.rs" | "main.rs") {
        return false;
    }
    if file_name == "mod.rs" {
        return true;
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return false;
    }
    path.with_extension("").is_dir()
}

fn root_module_name(path: &Path) -> Option<String> {
    if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
        return path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned);
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToOwned::to_owned)
}

fn resolve_parent_visibility(
    path: &Path,
    module_name: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Option<ParentModuleVisibility> {
    for candidate in parent_candidates(path) {
        let Some(text) = file_texts.get(&candidate) else {
            continue;
        };
        let Ok(file) = syn::parse_file(text) else {
            continue;
        };
        if let Some(item_mod) = find_top_level_module(&file, module_name) {
            return Some(classify_parent_visibility(item_mod));
        }
    }
    None
}

fn parent_candidates(path: &Path) -> Vec<PathBuf> {
    let Some(parent_dir) = path.parent().map(Path::to_path_buf) else {
        return Vec::new();
    };
    let module_parent_dir = if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
        let Some(module_dir_parent) = parent_dir.parent() else {
            return Vec::new();
        };
        module_dir_parent.to_path_buf()
    } else {
        parent_dir
    };

    let mut candidates = Vec::new();
    for candidate in [
        module_parent_dir.join("lib.rs"),
        module_parent_dir.join("main.rs"),
        module_parent_dir.with_extension("rs"),
        module_parent_dir.join("mod.rs"),
    ] {
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn find_top_level_module<'a>(file: &'a File, module_name: &str) -> Option<&'a ItemMod> {
    file.items.iter().find_map(|item| match item {
        Item::Mod(item_mod) if item_mod.ident == module_name => Some(item_mod),
        _ => None,
    })
}

fn classify_parent_visibility(item_mod: &ItemMod) -> ParentModuleVisibility {
    match &item_mod.vis {
        Visibility::Inherited => ParentModuleVisibility::Internal("mod".to_string()),
        Visibility::Public(_) => ParentModuleVisibility::Public,
        Visibility::Restricted(restricted) => {
            ParentModuleVisibility::Internal(render_restricted_visibility(restricted))
        }
    }
}

fn render_restricted_visibility(restricted: &syn::VisRestricted) -> String {
    if restricted.in_token.is_some() {
        return format!("pub(in {})", render_syn_path(&restricted.path));
    }
    format!("pub({})", render_syn_path(&restricted.path))
}

fn render_syn_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn collect_child_modules(file: &File) -> BTreeSet<String> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(item_mod) if item_mod.content.is_none() => Some(item_mod.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn collect_visible_owner_modules(
    file: &File,
    child_modules: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut visible_modules = BTreeSet::new();

    for item in &file.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        if matches!(item_use.vis, Visibility::Inherited) {
            continue;
        }

        visible_modules.extend(
            collect_reexports(item_use)
                .into_iter()
                .filter_map(|segments| child_source_module(&segments, child_modules)),
        );
    }

    visible_modules
}

fn collect_reexports(item_use: &ItemUse) -> Vec<Vec<String>> {
    let mut exports = Vec::new();
    collect_use_tree_segments(&item_use.tree, &mut Vec::new(), &mut exports);
    exports
}

fn collect_use_tree_segments(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    exports: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree_segments(&path.tree, prefix, exports);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let mut segments = prefix.clone();
            segments.push(name.ident.to_string());
            exports.push(segments);
        }
        UseTree::Rename(rename) => {
            let mut segments = prefix.clone();
            segments.push(rename.ident.to_string());
            exports.push(segments);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_segments(item, prefix, exports);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn child_source_module(segments: &[String], child_modules: &BTreeSet<String>) -> Option<String> {
    segments
        .iter()
        .filter(|segment| !matches!(segment.as_str(), "self" | "crate" | "super"))
        .find(|segment| child_modules.contains(segment.as_str()))
        .cloned()
}

fn collect_root_doc(file: &File) -> Option<RootDoc> {
    let doc_attrs = file
        .attrs
        .iter()
        .filter(|attr| matches!(attr.style, AttrStyle::Inner(_)) && attr.path().is_ident("doc"))
        .collect::<Vec<_>>();
    let first_attr = doc_attrs.first()?;
    let line_number = span_start(first_attr.span()).line;
    let text = doc_attrs
        .into_iter()
        .filter_map(extract_doc_text)
        .collect::<Vec<_>>()
        .join(" ");
    if text.trim().is_empty() {
        return None;
    }
    Some(RootDoc { line_number, text })
}

fn extract_doc_text(attr: &syn::Attribute) -> Option<String> {
    let Meta::NameValue(name_value) = &attr.meta else {
        return None;
    };
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = &name_value.value
    else {
        return None;
    };
    Some(lit_str.value())
}

fn child_module_positions(text: &str, child_modules: &BTreeSet<String>) -> BTreeMap<String, usize> {
    let doc_text = text.to_lowercase();
    child_modules
        .iter()
        .filter_map(|module_name| {
            find_token_position(&doc_text, &module_name.to_lowercase())
                .map(|position| (module_name.clone(), position))
        })
        .collect()
}

fn find_token_position(text: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }

    text.match_indices(needle).find_map(|(start, _)| {
        let end = start + needle.len();
        is_token_boundary(text, start, end).then_some(start)
    })
}

fn is_token_boundary(text: &str, start: usize, end: usize) -> bool {
    let left_is_boundary = text[..start]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_alphanumeric() && character != '_');
    let right_is_boundary = text[end..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric() && character != '_');
    left_is_boundary && right_is_boundary
}

fn first_inline_owner_hint(text: &str) -> Option<String> {
    let mut in_code = false;
    let mut current = String::new();

    for character in text.chars() {
        if character == '`' {
            if in_code {
                let hint = current.trim();
                if !hint.is_empty() {
                    return Some(hint.to_string());
                }
                current.clear();
                in_code = false;
            } else {
                current.clear();
                in_code = true;
            }
            continue;
        }

        if in_code {
            current.push(character);
        }
    }

    None
}

fn inline_owner_hint_matches_module(hinted_owner: &str, visible_owner: &str) -> bool {
    let normalized_visible_owner = visible_owner.to_lowercase();
    hinted_owner
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .any(|segment| segment.eq_ignore_ascii_case(&normalized_visible_owner))
}

fn span_start(span: Span) -> SourceLocation {
    let LineColumn { line, .. } = span.start();
    SourceLocation { line: line.max(1) }
}

#[derive(Debug, Clone)]
struct RootDoc {
    line_number: usize,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParentModuleVisibility {
    Public,
    Internal(String),
}

impl ParentModuleVisibility {
    fn rendered_declaration(&self, module_name: &str) -> String {
        match self {
            Self::Public => format!("pub mod {module_name};"),
            Self::Internal(prefix) => format!("{prefix} {module_name};"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceLocation {
    line: usize,
}
