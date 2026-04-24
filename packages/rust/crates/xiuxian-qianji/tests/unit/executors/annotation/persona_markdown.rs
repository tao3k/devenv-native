use super::{persona_profile_from_markdown, strip_persona_suffix};

#[test]
fn strip_persona_suffix_trims_common_suffixes() {
    assert_eq!(
        strip_persona_suffix("Agenda Steward Persona"),
        "Agenda Steward"
    );
    assert_eq!(
        strip_persona_suffix("Agenda Steward persona"),
        "Agenda Steward"
    );
    assert_eq!(strip_persona_suffix("Agenda Steward"), "Agenda Steward");
}

#[test]
fn persona_profile_from_markdown_prefers_frontmatter_title_and_extracts_sections() {
    let markdown = r#"---
title: Agenda Steward Persona
metadata:
  routing_keywords: ["schedule", "calendar"]
  intents: ["planning"]
---

# Ignored Heading

Operating profile:
- Calm and grounded.
- Turns ambiguity into next actions.

Behavior contract:
- Refuse impossible schedules.
- Prefer verifiable commitments.
"#;

    let profile = persona_profile_from_markdown(
        "wendao://skills/agenda-management/references/steward.md",
        markdown,
    );

    assert_eq!(profile.id, "agenda_steward");
    assert_eq!(profile.name, "Agenda Steward");
    assert_eq!(
        profile.voice_tone,
        "Calm and grounded. Turns ambiguity into next actions."
    );
    assert_eq!(
        profile.guidelines,
        vec![
            "Refuse impossible schedules.".to_string(),
            "Prefer verifiable commitments.".to_string()
        ]
    );
    assert_eq!(
        profile.style_anchors,
        vec![
            "schedule".to_string(),
            "calendar".to_string(),
            "planning".to_string()
        ]
    );
    assert_eq!(
        profile.metadata.get("source_uri"),
        Some(&"wendao://skills/agenda-management/references/steward.md".to_string())
    );
}
