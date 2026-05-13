use std::collections::HashMap;
use std::path::Path;
use xiuxian_qianhuan::persona::PersonaProfile;
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_wendao_parsers::frontmatter::parse_frontmatter;

pub(super) fn persona_profile_from_markdown(uri: &str, markdown: &str) -> PersonaProfile {
    let frontmatter = parse_frontmatter(markdown);
    let background = strip_markdown_frontmatter(markdown);
    let heading_title = extract_markdown_h1(background.as_str());
    let parsed_uri = WendaoResourceUri::parse(uri).ok();
    let fallback_name = parsed_uri
        .as_ref()
        .and_then(|value| {
            Path::new(value.entity_name())
                .file_stem()
                .and_then(|stem| stem.to_str())
        })
        .map_or_else(|| "Persona".to_string(), humanize_identifier);
    let persona_name = frontmatter
        .title
        .as_deref()
        .or(heading_title.as_deref())
        .map(strip_persona_suffix)
        .filter(|value: &&str| !value.trim().is_empty())
        .map_or(fallback_name, ToString::to_string);
    let persona_id = persona_id_from_name(persona_name.as_str());
    let operating_profile = extract_section_bullets(background.as_str(), "Operating profile");
    let behavior_contract = extract_section_bullets(background.as_str(), "Behavior contract");
    let mut style_anchors = frontmatter.routing_keywords;
    style_anchors.extend(frontmatter.intents);
    style_anchors = dedup_non_empty(style_anchors);

    let mut metadata = HashMap::new();
    metadata.insert("source_uri".to_string(), uri.to_string());

    PersonaProfile {
        id: persona_id,
        name: persona_name,
        voice_tone: if operating_profile.is_empty() {
            "Calm, practical, and context-grounded.".to_string()
        } else {
            operating_profile.join(" ")
        },
        background: Some(background),
        guidelines: if behavior_contract.is_empty() {
            vec!["Respond with concise and actionable guidance.".to_string()]
        } else {
            behavior_contract
        },
        style_anchors,
        cot_template:
            "Extract constraints, reason about feasibility, then produce one executable output."
                .to_string(),
        forbidden_words: Vec::new(),
        metadata,
    }
}

fn strip_persona_suffix(raw: &str) -> &str {
    let trimmed = raw.trim();
    trimmed
        .strip_suffix(" Persona")
        .or_else(|| trimmed.strip_suffix(" persona"))
        .unwrap_or(trimmed)
        .trim()
}

fn strip_markdown_frontmatter(markdown: &str) -> String {
    let normalized = markdown.replace("\r\n", "\n");
    if let Some(rest) = normalized.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        return rest[end + "\n---\n".len()..].trim().to_string();
    }
    normalized.trim().to_string()
}

fn extract_section_bullets(content: &str, heading: &str) -> Vec<String> {
    let heading_token = format!("{}:", heading.trim().to_ascii_lowercase());
    let mut in_section = false;
    let mut bullets = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !in_section {
            if trimmed.to_ascii_lowercase() == heading_token {
                in_section = true;
            }
            continue;
        }
        if trimmed.starts_with('#') {
            break;
        }
        if trimmed.ends_with(':') && !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            break;
        }
        if let Some(value) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            let value = value.trim();
            if !value.is_empty() {
                bullets.push(value.to_string());
            }
            continue;
        }
        if trimmed.is_empty() {
            if !bullets.is_empty() {
                break;
            }
            continue;
        }
        if bullets.is_empty() {
            bullets.push(trimmed.to_string());
        } else {
            break;
        }
    }
    bullets
}

fn extract_markdown_h1(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn humanize_identifier(identifier: &str) -> String {
    let mut out = String::new();
    for (index, part) in identifier
        .split(['-', '_'])
        .filter(|value| !value.trim().is_empty())
        .enumerate()
    {
        if index > 0 {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        "Persona".to_string()
    } else {
        out
    }
}

fn persona_id_from_name(name: &str) -> String {
    let mut id = String::new();
    let mut previous_was_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !id.is_empty() {
            id.push('_');
            previous_was_separator = true;
        }
    }
    id.trim_matches('_').to_string()
}

fn dedup_non_empty(values: Vec<String>) -> Vec<String> {
    values.into_iter().fold(Vec::new(), |mut out, value| {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !out.iter().any(|existing| existing == trimmed) {
            out.push(trimmed.to_string());
        }
        out
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/executors/annotation/persona_markdown.rs"]
mod tests;
