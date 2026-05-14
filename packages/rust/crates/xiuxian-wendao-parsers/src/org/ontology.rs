//! Org-native ontology authoring DTO compilation.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use orgize::Org;
use orgize::rowan::ast::AstNode;
use orgize::syntax_ast::Headline;
use serde::{Deserialize, Serialize};

use super::document::parse_org_document;

/// Draft schema id for Org ontology authoring DTOs.
pub const ORG_ONTOLOGY_AUTHORING_SCHEMA_ID: &str = "xiuxian_wendao.org_ontology_authoring.v0.draft";

/// Parser-owned Org ontology authoring document DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgOntologyAuthoringDocument {
    /// Draft schema identifier for the compiled DTO shape.
    pub schema: String,
    /// Stable document identifier from the Org document metadata or content hash.
    pub document_id: String,
    /// Repository-relative source path or caller-provided source identity.
    pub source_path: String,
    /// Content hash of the source Org document.
    pub source_hash: String,
    /// Compiled ontology authoring sections.
    pub sections: Vec<OrgOntologyAuthoringSection>,
}

/// Parser-owned Org ontology authoring section DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgOntologyAuthoringSection {
    /// Stable section identifier from the property drawer or content hash.
    pub section_id: String,
    /// Heading path as an ordered vector of Org headings.
    pub heading_path: Vec<String>,
    /// Org heading level.
    pub level: usize,
    /// Leaf heading title.
    pub title: String,
    /// Ontology authoring kind.
    pub authoring_kind: OrgOntologyAuthoringKind,
    /// Ontology authoring lifecycle state.
    pub lifecycle_state: OrgOntologyLifecycleState,
    /// Org heading tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Property drawer values normalized to upper-case keys.
    pub properties: BTreeMap<String, String>,
    /// Table projections. Reserved for the next parser slice.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<OrgOntologyAuthoringTable>,
    /// Embedded artifact projections. Reserved for the next parser slice.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embedded_artifacts: Vec<OrgOntologyEmbeddedArtifact>,
    /// Reopenable source span for the originating Org section.
    pub source_span: OrgOntologySourceSpan,
}

/// Parser-owned Org table projection DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgOntologyAuthoringTable {
    /// Table name or parser-generated identifier.
    pub name: String,
    /// Table purpose within the ontology authoring contract.
    pub kind: OrgOntologyTableKind,
    /// Table column names.
    pub columns: Vec<String>,
    /// Table rows represented as string maps.
    pub rows: Vec<BTreeMap<String, String>>,
    /// Reopenable source span for the source table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_span: Option<OrgOntologySourceSpan>,
}

/// Typed catalog value for ontology authoring section kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrgOntologyAuthoringKind(String);

impl OrgOntologyAuthoringKind {
    /// Returns the stable string value used by parser DTO JSON.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq<&str> for OrgOntologyAuthoringKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Typed catalog value for ontology authoring lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrgOntologyLifecycleState(String);

impl OrgOntologyLifecycleState {
    /// Returns the stable string value used by parser DTO JSON.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq<&str> for OrgOntologyLifecycleState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Typed catalog value for ontology authoring table kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrgOntologyTableKind(String);

impl OrgOntologyTableKind {
    /// Returns the stable string value used by parser DTO JSON.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq<&str> for OrgOntologyTableKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Parser-owned source-block or generated preview artifact DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgOntologyEmbeddedArtifact {
    /// Source block language.
    pub language: String,
    /// Embedded artifact purpose.
    pub purpose: String,
    /// Hash of the embedded artifact body.
    pub content_hash: String,
    /// Reopenable source span for the source block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_span: Option<OrgOntologySourceSpan>,
}

/// Reopenable source span for parser-owned Org ontology DTOs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgOntologySourceSpan {
    /// Inclusive 1-based start line.
    pub start_line: usize,
    /// Inclusive 1-based start column.
    pub start_column: usize,
    /// Inclusive 1-based end line.
    pub end_line: usize,
    /// Inclusive 1-based end column.
    pub end_column: usize,
}

/// Parser-owned Org ontology authoring compile error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrgOntologyAuthoringError {
    /// The document does not contain any typed ontology authoring section.
    EmptyAuthoringDocument,
    /// A section is missing a required ontology authoring kind.
    MissingAuthoringKind {
        /// Org heading path.
        heading_path: String,
        /// Source span for the offending section.
        source_span: OrgOntologySourceSpan,
    },
    /// A section contains an unsupported ontology authoring kind.
    UnsupportedAuthoringKind {
        /// Org heading path.
        heading_path: String,
        /// Raw property value.
        value: String,
        /// Source span for the offending section.
        source_span: OrgOntologySourceSpan,
    },
    /// A section contains an unsupported lifecycle state.
    UnsupportedLifecycleState {
        /// Org heading path.
        heading_path: String,
        /// Raw property or TODO value.
        value: String,
        /// Source span for the offending section.
        source_span: OrgOntologySourceSpan,
    },
}

impl Display for OrgOntologyAuthoringError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyAuthoringDocument => {
                write!(formatter, "Org ontology authoring document has no sections")
            }
            Self::MissingAuthoringKind {
                heading_path,
                source_span,
            } => write!(
                formatter,
                "Org ontology section `{heading_path}` at line {} is missing AUTHORING_KIND, ONTOLOGY_KIND, or KIND",
                source_span.start_line
            ),
            Self::UnsupportedAuthoringKind {
                heading_path,
                value,
                source_span,
            } => write!(
                formatter,
                "Org ontology section `{heading_path}` at line {} has unsupported authoring kind `{value}`",
                source_span.start_line
            ),
            Self::UnsupportedLifecycleState {
                heading_path,
                value,
                source_span,
            } => write!(
                formatter,
                "Org ontology section `{heading_path}` at line {} has unsupported lifecycle state `{value}`",
                source_span.start_line
            ),
        }
    }
}

impl Error for OrgOntologyAuthoringError {}

/// Compile native Org ontology authoring content into a parser-owned DTO.
///
/// # Errors
///
/// Returns [`OrgOntologyAuthoringError`] when the Org document has no headings,
/// when a heading has no ontology authoring kind, or when authoring kind /
/// lifecycle values are outside the draft parser contract vocabulary.
pub fn compile_org_ontology_authoring_document(
    content: &str,
    source_path: impl Into<String>,
) -> Result<OrgOntologyAuthoringDocument, OrgOntologyAuthoringError> {
    let source_path = source_path.into();
    let source_hash = content_hash(content);
    let document = parse_org_document(content, source_path.as_str());
    let document_id = document
        .raw_metadata
        .as_ref()
        .and_then(|metadata| {
            metadata
                .properties
                .get("ID")
                .or_else(|| metadata.properties.get("DOCUMENT_ID"))
                .cloned()
        })
        .unwrap_or_else(|| format!("org-authoring:{}", short_hash(source_hash.as_str())));

    let org = Org::parse(content);
    let (_, sections) = org
        .syntax_document()
        .syntax()
        .descendants()
        .filter_map(Headline::cast)
        .try_fold(
            (
                Vec::<String>::new(),
                Vec::<OrgOntologyAuthoringSection>::new(),
            ),
            |(mut heading_stack, mut sections), headline| {
                let compiled = compile_headline(content, &mut heading_stack, &headline)?;
                sections.push(compiled);
                Ok::<_, OrgOntologyAuthoringError>((heading_stack, sections))
            },
        )?;

    if sections.is_empty() {
        return Err(OrgOntologyAuthoringError::EmptyAuthoringDocument);
    }

    Ok(OrgOntologyAuthoringDocument {
        schema: ORG_ONTOLOGY_AUTHORING_SCHEMA_ID.to_string(),
        document_id,
        source_path,
        source_hash,
        sections,
    })
}

fn compile_headline(
    content: &str,
    heading_stack: &mut Vec<String>,
    headline: &Headline,
) -> Result<OrgOntologyAuthoringSection, OrgOntologyAuthoringError> {
    let title = headline.title_raw().trim().to_string();
    if heading_stack.len() >= headline.level() {
        heading_stack.truncate(headline.level().saturating_sub(1));
    }
    heading_stack.push(title.clone());
    let heading_path = heading_stack.clone();
    let heading_path_label = heading_path.join(" / ");
    let source_span = source_span_for_headline(content, headline);
    let properties = headline_properties(headline);
    let authoring_kind = authoring_kind(&properties).ok_or_else(|| {
        OrgOntologyAuthoringError::MissingAuthoringKind {
            heading_path: heading_path_label.clone(),
            source_span: source_span.clone(),
        }
    })?;
    let authoring_kind = normalize_authoring_kind(&authoring_kind).ok_or_else(|| {
        OrgOntologyAuthoringError::UnsupportedAuthoringKind {
            heading_path: heading_path_label.clone(),
            value: authoring_kind.clone(),
            source_span: source_span.clone(),
        }
    })?;
    let lifecycle_state = lifecycle_state(headline, &properties).ok_or_else(|| {
        OrgOntologyAuthoringError::UnsupportedLifecycleState {
            heading_path: heading_path_label.clone(),
            value: lifecycle_source_value(headline, &properties),
            source_span: source_span.clone(),
        }
    })?;

    Ok(OrgOntologyAuthoringSection {
        section_id: section_id(&heading_path_label, &properties),
        heading_path,
        level: headline.level(),
        title,
        authoring_kind: OrgOntologyAuthoringKind(authoring_kind),
        lifecycle_state: OrgOntologyLifecycleState(lifecycle_state),
        tags: headline.tags().map(|tag| tag.to_string()).collect(),
        properties,
        tables: extract_section_tables(content, headline),
        embedded_artifacts: extract_section_embedded_artifacts(content, headline),
        source_span,
    })
}

fn headline_properties(headline: &Headline) -> BTreeMap<String, String> {
    headline
        .properties()
        .map(|drawer| {
            drawer
                .iter()
                .filter_map(|(key, value)| normalize_property_pair(key.as_ref(), value.as_ref()))
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_property_pair(key: &str, value: &str) -> Option<(String, String)> {
    let key = key.trim().to_uppercase();
    let value = value.trim().to_string();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key, value))
    }
}

fn authoring_kind(properties: &BTreeMap<String, String>) -> Option<String> {
    properties
        .get("AUTHORING_KIND")
        .or_else(|| properties.get("ONTOLOGY_KIND"))
        .or_else(|| properties.get("KIND"))
        .cloned()
}

fn normalize_authoring_kind(value: &str) -> Option<String> {
    normalize_token(value).and_then(|value| match value.as_str() {
        "domain" => Some("domain".to_string()),
        "object" | "object_type" => Some("object_type".to_string()),
        "link" | "relation" | "link_type" | "relation_type" => Some("link_type".to_string()),
        "action" | "action_type" => Some("action_type".to_string()),
        "query" | "query_type" => Some("query_type".to_string()),
        "interface" | "interface_type" => Some("interface_type".to_string()),
        "value" | "value_type" => Some("value_type".to_string()),
        "validation" | "validation_rule" => Some("validation_rule".to_string()),
        "dataset" | "dataset_mapping" | "data_mapping" => Some("dataset_mapping".to_string()),
        _ => None,
    })
}

fn lifecycle_state(headline: &Headline, properties: &BTreeMap<String, String>) -> Option<String> {
    properties
        .get("LIFECYCLE_STATE")
        .or_else(|| properties.get("STATUS"))
        .or_else(|| properties.get("STATE"))
        .and_then(|value| normalize_lifecycle_state(value))
        .or_else(|| {
            headline
                .todo_keyword()
                .and_then(|keyword| normalize_lifecycle_state(keyword.as_ref()))
        })
        .or_else(|| Some("draft".to_string()))
}

fn lifecycle_source_value(headline: &Headline, properties: &BTreeMap<String, String>) -> String {
    properties
        .get("LIFECYCLE_STATE")
        .or_else(|| properties.get("STATUS"))
        .or_else(|| properties.get("STATE"))
        .cloned()
        .or_else(|| headline.todo_keyword().map(|keyword| keyword.to_string()))
        .unwrap_or_else(|| "draft".to_string())
}

fn normalize_lifecycle_state(value: &str) -> Option<String> {
    normalize_token(value).and_then(|value| match value.as_str() {
        "todo" | "draft" => Some("draft".to_string()),
        "candidate" => Some("candidate".to_string()),
        "doing" | "review" | "in_review" => Some("review".to_string()),
        "done" | "accepted" => Some("accepted".to_string()),
        "cancelled" | "canceled" | "retired" => Some("retired".to_string()),
        _ => None,
    })
}

fn normalize_token(value: &str) -> Option<String> {
    let normalized = value.trim().replace(['-', ' '], "_").to_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn section_id(heading_path_label: &str, properties: &BTreeMap<String, String>) -> String {
    properties
        .get("ID")
        .or_else(|| properties.get("SECTION_ID"))
        .cloned()
        .unwrap_or_else(|| {
            let payload = format!("{heading_path_label}\n{}", content_hash(heading_path_label));
            format!(
                "section:{}",
                short_hash(content_hash(payload.as_str()).as_str())
            )
        })
}

fn source_span_for_headline(content: &str, headline: &Headline) -> OrgOntologySourceSpan {
    let start = usize::from(headline.start()).min(content.len());
    let end = usize::from(headline.end()).min(content.len()).max(start);
    let start_position = line_column_for_byte(content, start);
    let end_position = line_column_for_byte(content, end.saturating_sub(1).max(start));
    OrgOntologySourceSpan {
        start_line: start_position.0,
        start_column: start_position.1,
        end_line: end_position.0.max(start_position.0),
        end_column: end_position.1,
    }
}

fn extract_section_tables(content: &str, headline: &Headline) -> Vec<OrgOntologyAuthoringTable> {
    let lines = immediate_section_lines(content, headline);
    let mut tables = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let (line_number, line) = &lines[index];
        if !line.trim_start().starts_with('|') {
            index += 1;
            continue;
        }

        let start_index = index;
        while index < lines.len() && lines[index].1.trim_start().starts_with('|') {
            index += 1;
        }
        let table_lines = &lines[start_index..index];
        if let Some(table) = compile_table(table_lines, *line_number) {
            tables.push(table);
        }
    }

    tables
}

fn extract_section_embedded_artifacts(
    content: &str,
    headline: &Headline,
) -> Vec<OrgOntologyEmbeddedArtifact> {
    let lines = immediate_section_lines(content, headline);
    let mut artifacts = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let (line_number, line) = &lines[index];
        let trimmed = line.trim();
        let Some(language) = source_block_language(trimmed) else {
            index += 1;
            continue;
        };

        let start_line = *line_number;
        let mut body = Vec::<String>::new();
        index += 1;
        let mut end_line = start_line;
        while index < lines.len() {
            let (current_line_number, current_line) = &lines[index];
            end_line = *current_line_number;
            if current_line.trim().eq_ignore_ascii_case("#+END_SRC") {
                break;
            }
            body.push(current_line.clone());
            index += 1;
        }
        artifacts.push(OrgOntologyEmbeddedArtifact {
            language: language.clone(),
            purpose: source_block_purpose(&language, trimmed),
            content_hash: content_hash(body.join("\n").as_str()),
            source_span: Some(OrgOntologySourceSpan {
                start_line,
                start_column: 1,
                end_line,
                end_column: lines
                    .iter()
                    .find(|(candidate_line, _)| *candidate_line == end_line)
                    .map_or(1, |(_, value)| value.len().max(1)),
            }),
        });
        index += 1;
    }

    artifacts
}

fn compile_table(
    table_lines: &[(usize, String)],
    start_line: usize,
) -> Option<OrgOntologyAuthoringTable> {
    let parsed_rows: Vec<Vec<String>> = table_lines
        .iter()
        .map(|(_, line)| parse_org_table_row(line))
        .filter(|cells| !cells.is_empty() && !is_separator_row(cells))
        .collect();
    let (columns, rows) = parsed_rows.split_first()?;
    let row_maps = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .cloned()
                .zip(row.iter().cloned())
                .collect::<BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();

    let kind = table_kind_for_columns(columns);
    Some(OrgOntologyAuthoringTable {
        name: table_name_for_kind(kind.as_str()),
        kind,
        columns: columns.clone(),
        rows: row_maps,
        source_span: Some(OrgOntologySourceSpan {
            start_line,
            start_column: 1,
            end_line: table_lines
                .last()
                .map_or(start_line, |(line_number, _)| *line_number),
            end_column: table_lines.last().map_or(1, |(_, line)| line.len().max(1)),
        }),
    })
}

fn immediate_section_lines(content: &str, headline: &Headline) -> Vec<(usize, String)> {
    let start = usize::from(headline.start()).min(content.len());
    let end = usize::from(headline.end()).min(content.len()).max(start);
    let start_line = line_column_for_byte(content, start).0;
    let section = &content[start..end];
    let mut lines = Vec::new();
    for (offset, line) in section.lines().enumerate().skip(1) {
        if is_headline_line(line) {
            break;
        }
        lines.push((start_line + offset, line.to_string()));
    }
    lines
}

fn parse_org_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn is_separator_row(cells: &[String]) -> bool {
    cells.iter().all(|cell| {
        !cell.is_empty()
            && cell
                .chars()
                .all(|character| matches!(character, '-' | '+' | ' '))
    })
}

fn table_name_for_kind(kind: &str) -> String {
    match kind {
        "dataset_columns" => "Dataset Columns",
        "object_mapping" => "Object Mapping",
        "link_mapping" => "Link Mapping",
        "mapping_evidence" => "Mapping Evidence",
        other => other,
    }
    .to_string()
}

fn table_kind_for_columns(columns: &[String]) -> OrgOntologyTableKind {
    let normalized = columns
        .iter()
        .map(|column| normalize_token(column).unwrap_or_default())
        .collect::<Vec<_>>();
    let kind = if normalized.contains(&"ontology_object_type".to_string())
        || normalized.contains(&"rdf_class".to_string())
    {
        "object_mapping"
    } else if normalized.contains(&"predicate".to_string())
        || normalized.contains(&"rdf_property".to_string())
    {
        "link_mapping"
    } else if normalized.contains(&"evidence_id".to_string()) {
        "mapping_evidence"
    } else if normalized.contains(&"source_table".to_string())
        && normalized.contains(&"required_columns".to_string())
    {
        "dataset_columns"
    } else {
        "evidence"
    };
    OrgOntologyTableKind(kind.to_string())
}

fn source_block_language(trimmed_line: &str) -> Option<String> {
    let upper = trimmed_line.to_ascii_uppercase();
    if !upper.starts_with("#+BEGIN_SRC") {
        return None;
    }
    trimmed_line
        .split_whitespace()
        .nth(1)
        .map(|language| language.trim().to_lowercase())
        .filter(|language| !language.is_empty())
}

fn source_block_purpose(language: &str, begin_line: &str) -> String {
    if begin_line.contains(":purpose mapping") || language == "sql" {
        "mapping".to_string()
    } else {
        "note".to_string()
    }
}

fn is_headline_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let stars = trimmed
        .chars()
        .take_while(|character| *character == '*')
        .count();
    stars > 0 && trimmed.chars().nth(stars).is_some_and(char::is_whitespace)
}

fn line_column_for_byte(content: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(content.len());
    let (line, line_start) = content.bytes().enumerate().take(offset).fold(
        (1usize, 0usize),
        |(line, line_start), (index, byte)| {
            if byte == b'\n' {
                (line + 1, index + 1)
            } else {
                (line, line_start)
            }
        },
    );
    (line, offset.saturating_sub(line_start) + 1)
}

fn content_hash(content: &str) -> String {
    format!("blake3:{}", blake3::hash(content.as_bytes()).to_hex())
}

fn short_hash(hash: &str) -> String {
    hash.rsplit(':')
        .next()
        .unwrap_or(hash)
        .chars()
        .take(16)
        .collect()
}
