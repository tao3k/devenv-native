//! `enhancer::resource_semantics` owns Wendao enhancer resource semantics behavior.

use std::sync::Arc;

use crate::entity::{EntityType, RelationType};

/// Structural semantics classification for a skill reference link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillReferenceSemantics {
    /// Recommended entity type for the target reference.
    pub entity: EntityType,
    /// Recommended relation type from skill to this reference.
    pub relation: RelationType,
    /// Semantic reference category (e.g. `template`, `knowledge`).
    pub reference_type: Option<Arc<str>>,
}

/// Classify a skill reference link based on its target path and explicit metadata.
#[must_use]
pub fn classify_skill_reference(
    explicit_type: Option<&str>,
    config_type: Option<&str>,
    entity_path: &str,
) -> SkillReferenceSemantics {
    let lower_path = entity_path.trim().to_ascii_lowercase();
    let explicit_reference_type = explicit_type.and_then(normalize_reference_type_label);
    let config_reference_type = config_type.and_then(normalize_reference_type_label);

    // 1. Explicit metadata categories take precedence.
    if let Some(reference_type) = explicit_reference_type {
        return semantics_for_reference_type(reference_type);
    }

    // 2. Config block type fallback.
    if let Some(reference_type) = config_reference_type {
        return semantics_for_reference_type(reference_type);
    }

    // 3. Path-based heuristics
    if lower_path.contains("/templates/") || lower_path.contains("/tpl/") {
        return semantics_for_reference_type("template");
    }
    if lower_path.contains("/personas/") {
        return semantics_for_reference_type("persona");
    }

    // 4. Attachment detection by file extension
    let attachment_extensions = [
        ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico", ".bmp", ".pdf", ".doc", ".docx",
        ".xls", ".xlsx", ".ppt", ".pptx", ".zip", ".tar", ".gz", ".rar", ".7z", ".mp3", ".mp4",
        ".wav", ".avi", ".mov", ".webm", ".ttf", ".otf", ".woff", ".woff2",
    ];
    for ext in &attachment_extensions {
        if lower_path.ends_with(ext) {
            return SkillReferenceSemantics {
                entity: EntityType::Other("Attachment".to_string()),
                relation: RelationType::AttachedTo,
                reference_type: Some(Arc::from("attachment")),
            };
        }
    }

    // Default fallback: generic relationship
    SkillReferenceSemantics {
        entity: EntityType::Other("Resource".to_string()),
        relation: RelationType::RelatedTo,
        reference_type: None,
    }
}

fn normalize_reference_type_label(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "template" | "tpl" | "prompt" => Some("template"),
        "persona" | "agent" => Some("persona"),
        "knowledge" | "doc" => Some("knowledge"),
        "workflow" | "flow" | "qianji-flow" => Some("qianji-flow"),
        _ => None,
    }
}

fn semantics_for_reference_type(reference_type: &str) -> SkillReferenceSemantics {
    match reference_type {
        "template" => SkillReferenceSemantics {
            entity: EntityType::Other("Template".to_string()),
            relation: RelationType::RelatedTo,
            reference_type: Some(Arc::from("template")),
        },
        "persona" => SkillReferenceSemantics {
            entity: EntityType::Other("Persona".to_string()),
            relation: RelationType::Manifests,
            reference_type: Some(Arc::from("persona")),
        },
        "knowledge" => SkillReferenceSemantics {
            entity: EntityType::Document,
            relation: RelationType::DocumentedIn,
            reference_type: Some(Arc::from("knowledge")),
        },
        "qianji-flow" => SkillReferenceSemantics {
            entity: EntityType::Other("QianjiFlow".to_string()),
            relation: RelationType::Governs,
            reference_type: Some(Arc::from("qianji-flow")),
        },
        _ => SkillReferenceSemantics {
            entity: EntityType::Other("Resource".to_string()),
            relation: RelationType::RelatedTo,
            reference_type: None,
        },
    }
}
