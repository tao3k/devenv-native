use std::collections::BTreeMap;

use serde::Serialize;

use crate::code_observation::extract_observations;
use crate::markdown_structure::{
    MarkdownStructuralItem, MarkdownStructure, parse_markdown_structure,
};

use super::types::MarkdownNote;

#[derive(Serialize)]
struct MarkdownNoteFingerprintPayload<'a> {
    title: &'a str,
    tags: &'a [String],
    doc_type: Option<&'a str>,
    references: Vec<ReferenceFingerprint<'a>>,
    targets: Vec<TargetFingerprint<'a>>,
    sections: Vec<SectionFingerprint<'a>>,
}

#[derive(Serialize)]
struct ReferenceFingerprint<'a> {
    kind: &'a str,
    target: Option<&'a str>,
    target_address: Option<&'a str>,
    original: &'a str,
}

#[derive(Serialize)]
struct TargetFingerprint<'a> {
    kind: &'a str,
    target: &'a str,
}

#[derive(Serialize)]
struct SectionFingerprint<'a> {
    heading_title: &'a str,
    heading_path: &'a str,
    heading_level: usize,
    section_text: &'a str,
    attributes: BTreeMap<&'a str, &'a str>,
    logbook: Vec<LogbookFingerprint<'a>>,
}

#[derive(Serialize)]
struct LogbookFingerprint<'a> {
    timestamp: &'a str,
    message: &'a str,
}

#[derive(Serialize)]
struct MarkdownSymbolFingerprintPayload {
    symbols: Vec<MarkdownSymbolFingerprint>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum MarkdownSymbolFingerprint {
    Section {
        level: usize,
        label: String,
    },
    Task {
        label: String,
    },
    Property {
        owner_title: Option<String>,
        key: String,
        value: String,
    },
    Observation {
        owner_title: Option<String>,
        raw_value: String,
    },
}

/// Compute a parser-owned semantic fingerprint for one Markdown note.
///
/// The fingerprint intentionally ignores raw body text, byte ranges, and line
/// ranges so metadata-only layout churn can reuse note-based incremental
/// search outputs while semantic Markdown note changes still invalidate them.
#[must_use]
pub fn fingerprint_markdown_note(note: &MarkdownNote) -> String {
    let payload = MarkdownNoteFingerprintPayload {
        title: note.document.core.title.as_str(),
        tags: note.document.core.tags.as_slice(),
        doc_type: note.document.core.doc_type.as_deref(),
        references: note
            .core
            .references
            .iter()
            .map(|reference| ReferenceFingerprint {
                kind: match reference.kind {
                    crate::references::MarkdownReferenceKind::Markdown => "markdown",
                    crate::references::MarkdownReferenceKind::WikiLink => "wiki_link",
                },
                target: reference.addressed_target.target.as_deref(),
                target_address: reference.addressed_target.target_address.as_deref(),
                original: reference.original.as_str(),
            })
            .collect(),
        targets: note
            .core
            .targets
            .iter()
            .map(|target| TargetFingerprint {
                kind: match target.kind {
                    crate::targets::MarkdownTargetOccurrenceKind::MarkdownLink => "markdown_link",
                    crate::targets::MarkdownTargetOccurrenceKind::MarkdownImage => "markdown_image",
                    crate::targets::MarkdownTargetOccurrenceKind::WikiLink => "wiki_link",
                    crate::targets::MarkdownTargetOccurrenceKind::WikiEmbed => "wiki_embed",
                },
                target: target.target.as_str(),
            })
            .collect(),
        sections: note
            .core
            .sections
            .iter()
            .map(|section| SectionFingerprint {
                heading_title: section.scope.heading_title.as_str(),
                heading_path: section.scope.heading_path.as_str(),
                heading_level: section.scope.heading_level,
                section_text: section.section_text.as_str(),
                attributes: section
                    .metadata
                    .attributes
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect(),
                logbook: section
                    .metadata
                    .logbook
                    .iter()
                    .map(|entry| LogbookFingerprint {
                        timestamp: entry.timestamp.as_str(),
                        message: entry.message.as_str(),
                    })
                    .collect(),
            })
            .collect(),
    };

    stable_payload_fingerprint("markdown_note", &payload)
}

/// Compute a parser-owned semantic fingerprint for Markdown local-symbol
/// surfaces.
///
/// The fingerprint is derived from parser-owned Markdown structure rather than
/// Wendao `AstSearchHit` payloads. It preserves the semantic surface that local
/// symbol indexing exposes today: headings, task items, property drawers, and
/// `:OBSERVE:` entries.
#[must_use]
pub fn fingerprint_markdown_symbol_surface(note: &MarkdownNote) -> String {
    let structure = parse_markdown_structure(note.document.core.body.as_str());
    fingerprint_markdown_symbol_surface_with_structure(note, &structure)
}

#[must_use]
pub(crate) fn fingerprint_markdown_symbol_surface_with_structure(
    note: &MarkdownNote,
    structure: &MarkdownStructure,
) -> String {
    let mut symbols = collect_markdown_structural_symbols(structure);
    for section in &note.core.sections {
        let owner_title = if !section.heading_path().trim().is_empty() {
            Some(section.heading_path().to_string())
        } else if !section.heading_title().trim().is_empty() {
            Some(section.heading_title().to_string())
        } else {
            None
        };

        let mut properties = section
            .metadata
            .attributes
            .iter()
            .filter(|(key, _)| !is_observation_attribute(key.as_str()))
            .map(|(key, value)| MarkdownSymbolFingerprint::Property {
                owner_title: owner_title.clone(),
                key: key.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();
        properties.sort_by(|left, right| match (left, right) {
            (
                MarkdownSymbolFingerprint::Property {
                    key: left_key,
                    value: left_value,
                    ..
                },
                MarkdownSymbolFingerprint::Property {
                    key: right_key,
                    value: right_value,
                    ..
                },
            ) => left_key.cmp(right_key).then(left_value.cmp(right_value)),
            _ => std::cmp::Ordering::Equal,
        });
        symbols.extend(properties);

        let mut observations = extract_observations(&section.metadata.attributes)
            .into_iter()
            .map(|observation| MarkdownSymbolFingerprint::Observation {
                owner_title: owner_title.clone(),
                raw_value: observation.raw_value,
            })
            .collect::<Vec<_>>();
        observations.sort_by(|left, right| match (left, right) {
            (
                MarkdownSymbolFingerprint::Observation {
                    raw_value: left_raw,
                    ..
                },
                MarkdownSymbolFingerprint::Observation {
                    raw_value: right_raw,
                    ..
                },
            ) => left_raw.cmp(right_raw),
            _ => std::cmp::Ordering::Equal,
        });
        symbols.extend(observations);
    }

    stable_payload_fingerprint(
        "markdown_symbol_surface",
        &MarkdownSymbolFingerprintPayload { symbols },
    )
}

fn stable_payload_fingerprint<T: Serialize + ?Sized>(kind: &str, value: &T) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_else(|error| {
        panic!("markdown note fingerprint payload should serialize: {error}");
    });
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&payload);
    hasher.finalize().to_hex().to_string()
}

fn collect_markdown_structural_symbols(
    structure: &MarkdownStructure,
) -> Vec<MarkdownSymbolFingerprint> {
    let mut symbols = Vec::new();

    for item in &structure.items {
        match item {
            MarkdownStructuralItem::Heading(heading) => {
                symbols.push(MarkdownSymbolFingerprint::Section {
                    level: heading.level,
                    label: heading.label.clone(),
                });
            }
            MarkdownStructuralItem::Task(task) => {
                symbols.push(MarkdownSymbolFingerprint::Task {
                    label: task.label.clone(),
                });
            }
        }
    }

    symbols
}

fn is_observation_attribute(key: &str) -> bool {
    key.trim().eq_ignore_ascii_case("OBSERVE")
}
