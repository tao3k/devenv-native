//! Modelica parsing bridged through the native parser-summary route.

use std::collections::BTreeMap;

use xiuxian_wendao_core::repo_intelligence::RegisteredRepository;
use xiuxian_wendao_core::repo_intelligence::{ImportKind, RepoIntelligenceError};

use super::parser_summary::fetch_modelica_parser_file_summary_blocking_for_repository;
use super::types::{ParsedDeclaration, ParsedImport};

/// Parse the package or class name from Modelica source through the native
/// parser-summary contract.
pub(crate) fn parse_package_name_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    contents: &str,
) -> Result<Option<String>, RepoIntelligenceError> {
    Ok(
        fetch_modelica_parser_file_summary_blocking_for_repository(
            repository, source_id, contents,
        )?
        .class_name,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageOverlayMetadata {
    pub(crate) package_name: String,
    pub(crate) imports: Vec<ParsedImport>,
    pub(crate) has_documentation_annotation: bool,
}

pub(crate) type RootPackageOverlayMetadata = PackageOverlayMetadata;

#[must_use]
pub(crate) fn parse_package_name_lexical(contents: &str) -> Option<String> {
    let stripped = strip_modelica_comments(contents);
    for line in stripped.lines() {
        let trimmed = trim_modelica_modifiers(line.trim());
        if let Some(rest) = trimmed.strip_prefix("package ") {
            return parse_modelica_identifier(rest);
        }
    }
    None
}

#[must_use]
pub(crate) fn parse_safe_root_package_overlay_metadata(
    contents: &str,
) -> Option<RootPackageOverlayMetadata> {
    let stripped = strip_modelica_comments(contents);
    let package_name = parse_package_name_from_stripped(stripped.as_str())?;
    if !package_overlay_is_safe(stripped.as_str()) {
        return None;
    }

    Some(PackageOverlayMetadata {
        package_name,
        imports: parse_imports_lexical_from_stripped(stripped.as_str()),
        has_documentation_annotation: contains_documentation_annotation(contents),
    })
}

#[must_use]
pub(crate) fn parse_safe_package_overlay_metadata(
    contents: &str,
    expected_package_name: &str,
) -> Option<PackageOverlayMetadata> {
    let metadata = parse_safe_root_package_overlay_metadata(contents)?;
    (metadata.package_name == expected_package_name).then_some(metadata)
}

/// Check if the source contains a Documentation annotation.
pub(crate) fn contains_documentation_annotation(contents: &str) -> bool {
    contents.contains("Documentation(")
}

/// Parse import statements from Modelica source through the native
/// parser-summary contract.
pub(crate) fn parse_imports_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    contents: &str,
) -> Result<Vec<ParsedImport>, RepoIntelligenceError> {
    Ok(
        fetch_modelica_parser_file_summary_blocking_for_repository(
            repository, source_id, contents,
        )?
        .imports,
    )
}

/// Parse symbol declarations from Modelica source through the native
/// parser-summary contract.
pub(crate) fn parse_symbol_declarations_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    contents: &str,
) -> Result<Vec<ParsedDeclaration>, RepoIntelligenceError> {
    Ok(
        fetch_modelica_parser_file_summary_blocking_for_repository(
            repository, source_id, contents,
        )?
        .declarations,
    )
}

fn strip_modelica_comments(contents: &str) -> String {
    let mut stripped = String::with_capacity(contents.len());
    let mut chars = contents.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                stripped.push('\n');
            }
            continue;
        }
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            } else if ch == '\n' {
                stripped.push('\n');
            }
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            in_line_comment = true;
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
            continue;
        }
        stripped.push(ch);
    }

    stripped
}

fn parse_package_name_from_stripped(stripped: &str) -> Option<String> {
    for line in stripped.lines() {
        let trimmed = trim_modelica_modifiers(line.trim());
        if let Some(rest) = trimmed.strip_prefix("package ") {
            return parse_modelica_identifier(rest);
        }
    }
    None
}

fn parse_modelica_identifier(raw: &str) -> Option<String> {
    let mut identifier = String::new();
    for ch in raw.chars() {
        if identifier.is_empty() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                identifier.push(ch);
                continue;
            }
            return None;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            identifier.push(ch);
            continue;
        }
        break;
    }
    if identifier.is_empty() {
        None
    } else {
        Some(identifier)
    }
}

fn trim_modelica_modifiers(mut line: &str) -> &str {
    loop {
        let Some(trimmed) = [
            line.strip_prefix("encapsulated "),
            line.strip_prefix("partial "),
            line.strip_prefix("final "),
        ]
        .into_iter()
        .flatten()
        .next() else {
            return line;
        };
        line = trimmed.trim_start();
    }
}

fn package_overlay_is_safe(stripped: &str) -> bool {
    let mut saw_package_header = false;
    for line in stripped.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let normalized = trim_modelica_modifiers(trimmed);
        if normalized.starts_with("within ") {
            continue;
        }
        if !saw_package_header {
            if normalized.starts_with("package ") {
                saw_package_header = true;
                continue;
            }
            return false;
        }
        if normalized.starts_with("end ") || normalized.starts_with("import ") {
            continue;
        }
        if normalized.starts_with("annotation(")
            || normalized.starts_with("annotation (")
            || trimmed.starts_with(')')
            || trimmed.starts_with(',')
        {
            continue;
        }
        if starts_with_unsupported_package_overlay_keyword(normalized) {
            return false;
        }
    }
    saw_package_header
}

fn starts_with_unsupported_package_overlay_keyword(line: &str) -> bool {
    [
        "model ",
        "class ",
        "record ",
        "block ",
        "connector ",
        "function ",
        "operator ",
        "type ",
        "extends ",
        "replaceable ",
        "redeclare ",
        "constant ",
        "parameter ",
        "equation",
        "algorithm",
        "initial equation",
        "initial algorithm",
        "package ",
    ]
    .iter()
    .any(|keyword| line.starts_with(keyword))
}

fn parse_imports_lexical_from_stripped(stripped: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    let mut statement = String::new();
    let mut statement_line_start = None;

    for (line_index, line) in stripped.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if statement.is_empty() && !trimmed.starts_with("import ") {
            continue;
        }
        if statement.is_empty() {
            statement_line_start = Some(line_index + 1);
        } else {
            statement.push(' ');
        }
        statement.push_str(trimmed);

        while let Some((complete, remainder)) = take_statement(statement.as_str()) {
            if let Some(parsed_import) =
                parse_import_statement(complete, statement_line_start.unwrap_or(line_index + 1))
            {
                imports.push(parsed_import);
            }
            statement = remainder.to_string();
            if statement.is_empty() {
                statement_line_start = None;
            }
        }
    }

    imports
}

fn take_statement(buffer: &str) -> Option<(&str, &str)> {
    let delimiter = buffer.find(';')?;
    Some((
        buffer[..delimiter].trim(),
        buffer[delimiter + 1..].trim_start(),
    ))
}

fn parse_import_statement(statement: &str, line_start: usize) -> Option<ParsedImport> {
    let body = statement.strip_prefix("import ")?.trim();
    if body.is_empty() {
        return None;
    }

    let mut attributes = BTreeMap::new();
    let (name, alias, kind) = if let Some((alias, target)) = body.split_once('=') {
        let alias = normalize_modelica_token(alias)?;
        let target = normalize_import_target(target)?;
        attributes.insert("dependency_alias".to_string(), alias.clone());
        attributes.insert("dependency_form".to_string(), "named_import".to_string());
        attributes.insert("dependency_local_name".to_string(), alias.clone());
        attributes.insert("dependency_target".to_string(), target.clone());
        (target, Some(alias), ImportKind::Module)
    } else if let Some(target) = body.strip_suffix(".*") {
        let target = normalize_import_target(target)?;
        let local_name = import_leaf_name(target.as_str());
        attributes.insert(
            "dependency_form".to_string(),
            "unqualified_import".to_string(),
        );
        attributes.insert("dependency_local_name".to_string(), local_name);
        attributes.insert("dependency_target".to_string(), target.clone());
        (target, None, ImportKind::Module)
    } else {
        let target = normalize_import_target(body)?;
        let local_name = import_leaf_name(target.as_str());
        attributes.insert(
            "dependency_form".to_string(),
            "qualified_import".to_string(),
        );
        attributes.insert("dependency_local_name".to_string(), local_name);
        attributes.insert("dependency_target".to_string(), target.clone());
        (target, None, ImportKind::Symbol)
    };

    Some(ParsedImport {
        name,
        alias,
        kind,
        line_start: Some(line_start),
        attributes,
    })
}

fn normalize_modelica_token(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<String>();
    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    Some(normalized)
}

fn normalize_import_target(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<String>();
    if normalized.is_empty() {
        return None;
    }
    let segments = normalized.split('.').collect::<Vec<_>>();
    if segments.is_empty()
        || segments.iter().any(|segment| {
            segment.is_empty()
                || !segment
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
    {
        return None;
    }
    Some(normalized)
}

fn import_leaf_name(import_path: &str) -> String {
    import_path
        .rsplit('.')
        .next()
        .unwrap_or(import_path)
        .trim()
        .to_string()
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/parsing.rs"]
mod tests;
