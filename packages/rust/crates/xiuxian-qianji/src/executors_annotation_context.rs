//! Executors annotation context surface for `xiuxian-qianji`.

use super::persona_markdown::persona_profile_from_markdown;
use crate::contracts::{
    FlowInstruction, NodeAnnotationExecutionMode, QianjiMechanism, QianjiOutput,
};
use crate::scheduler_preflight::{
    context_value_to_text, lookup_context_path, resolve_semantic_content,
    resolve_semantic_reference, resolve_wendao_uri_with_zhenfa,
};
use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;

const MAX_SNAPSHOT_COMPACTION_DEPTH: usize = 4;

#[derive(Debug, Clone)]
pub(super) struct PersonaProfile {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) voice_tone: String,
    pub(super) background: Option<String>,
    pub(super) guidelines: Vec<String>,
    pub(super) style_anchors: Vec<String>,
    pub(super) cot_template: String,
    pub(super) forbidden_words: Vec<String>,
    pub(super) metadata: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct PromptInjectionSnapshot {
    #[serde(default)]
    narrative_context: PromptInjectionNarrativeContext,
    #[serde(default)]
    working_history: String,
}

#[derive(Debug, Default, Deserialize)]
struct PromptInjectionNarrativeContext {
    #[serde(rename = "entry", default)]
    entries: Vec<String>,
}

/// Mechanism responsible for transmuting raw facts into persona-aligned context snapshots.
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct ContextAnnotator {
    /// Target persona ID defined in the registry.
    pub persona_id: String,
    /// Optional logical template target associated with this node.
    pub template_target: Option<String>,
    /// Context window behavior for this annotation node.
    pub execution_mode: NodeAnnotationExecutionMode,
    /// Whitelisted context keys that can be marshaled into narrative blocks.
    pub input_keys: Vec<String>,
    /// History key used when execution mode is `appended`.
    pub history_key: String,
    /// Context key where the rendered snapshot is stored.
    pub output_key: String,
}

impl ContextAnnotator {
    fn normalize_narrative_block(raw: &str) -> String {
        Self::normalize_narrative_block_with_depth(raw.trim(), 0)
    }

    fn normalize_narrative_block_with_depth(raw: &str, depth: usize) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty()
            || depth >= MAX_SNAPSHOT_COMPACTION_DEPTH
            || !trimmed.contains("<system_prompt_injection")
        {
            return trimmed.to_string();
        }

        Self::compact_prompt_snapshot(trimmed, depth).unwrap_or_else(|| trimmed.to_string())
    }

    fn compact_prompt_snapshot(raw: &str, depth: usize) -> Option<String> {
        let snapshot: PromptInjectionSnapshot = from_str(raw).ok()?;
        let mut segments = Vec::new();

        for entry in snapshot.narrative_context.entries {
            let normalized_entry =
                Self::normalize_narrative_block_with_depth(entry.as_str(), depth + 1);
            Self::push_compacted_segment(&mut segments, normalized_entry.as_str());
        }

        let normalized_history = Self::normalize_narrative_block_with_depth(
            snapshot.working_history.as_str(),
            depth + 1,
        );
        Self::push_compacted_segment(&mut segments, normalized_history.as_str());

        (!segments.is_empty()).then(|| segments.join("\n\n"))
    }

    fn push_compacted_segment(segments: &mut Vec<String>, segment: &str) {
        let trimmed = segment.trim();
        if trimmed.is_empty() || segments.iter().any(|existing| existing == trimmed) {
            return;
        }
        segments.push(trimmed.to_string());
    }

    fn collect_narrative_blocks(&self, context: &Value) -> Result<Vec<String>, String> {
        let mut blocks = Vec::new();
        for key in &self.input_keys {
            if key.trim_start().starts_with('$') {
                let text = resolve_semantic_content(key, context)?;
                if !text.trim().is_empty() {
                    let normalized_block = Self::normalize_narrative_block(text.as_str());
                    Self::push_compacted_segment(&mut blocks, normalized_block.as_str());
                }
                continue;
            }
            if let Some(value) = lookup_context_path(context, key)
                && let Some(text) = context_value_to_text(value)
            {
                let normalized_block = Self::normalize_narrative_block(text.as_str());
                Self::push_compacted_segment(&mut blocks, normalized_block.as_str());
            }
        }

        if blocks.is_empty() {
            match context.get("raw_facts") {
                Some(value) => {
                    if let Some(text) = context_value_to_text(value) {
                        let normalized_block = Self::normalize_narrative_block(text.as_str());
                        Self::push_compacted_segment(&mut blocks, normalized_block.as_str());
                    }
                }
                None => blocks.push(String::new()),
            }
        }

        Ok(blocks)
    }

    fn resolve_history_seed(&self, context: &Value) -> String {
        match self.execution_mode {
            NodeAnnotationExecutionMode::Isolated => String::new(),
            NodeAnnotationExecutionMode::Appended => context
                .get(&self.history_key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        }
    }

    fn metadata_key(&self, suffix: &str) -> String {
        if self.output_key == "annotated_prompt" {
            format!("annotated_{suffix}")
        } else {
            format!("{}_{}", self.output_key, suffix)
        }
    }

    fn merge_history_for_appended_mode(
        &self,
        current_history: &str,
        snapshot: &str,
    ) -> Option<String> {
        if self.execution_mode != NodeAnnotationExecutionMode::Appended {
            return None;
        }
        if current_history.is_empty() {
            return Some(snapshot.to_string());
        }
        Some(format!("{current_history}\n{snapshot}"))
    }

    fn resolve_persona_profile(persona_reference: &str) -> Result<PersonaProfile, String> {
        if persona_reference.trim_start().starts_with("wendao://") {
            return Self::resolve_wendao_persona_profile(persona_reference);
        }
        persona_profile_from_reference(persona_reference)
    }

    fn resolve_wendao_persona_profile(uri: &str) -> Result<PersonaProfile, String> {
        let canonical_uri = uri.trim();
        if canonical_uri.is_empty() {
            return Err("persona semantic URI must not be empty".to_string());
        }
        let markdown = resolve_wendao_uri_with_zhenfa(canonical_uri)?;
        Ok(persona_profile_from_markdown(
            canonical_uri,
            markdown.as_str(),
        ))
    }

    fn assemble_snapshot(persona: &PersonaProfile, blocks: &[String], history: &str) -> String {
        let mut snapshot = String::from("<system_prompt_injection>\n");
        push_xml_text_element(&mut snapshot, "persona_id", persona.id.as_str(), 1);
        push_xml_text_element(&mut snapshot, "persona_name", persona.name.as_str(), 1);
        push_xml_text_element(&mut snapshot, "voice_tone", persona.voice_tone.as_str(), 1);
        if let Some(background) = persona.background.as_deref() {
            push_xml_text_element(&mut snapshot, "background", background, 1);
        }
        push_xml_list_element(&mut snapshot, "guidelines", "item", &persona.guidelines, 1);
        push_xml_list_element(
            &mut snapshot,
            "style_anchors",
            "item",
            &persona.style_anchors,
            1,
        );
        push_xml_text_element(
            &mut snapshot,
            "cot_template",
            persona.cot_template.as_str(),
            1,
        );
        push_xml_list_element(
            &mut snapshot,
            "forbidden_words",
            "item",
            &persona.forbidden_words,
            1,
        );
        if !persona.metadata.is_empty() {
            snapshot.push_str("  <metadata>\n");
            let mut metadata = persona.metadata.iter().collect::<Vec<_>>();
            metadata.sort_by(|left, right| left.0.cmp(right.0));
            for (key, value) in metadata {
                snapshot.push_str("    <entry key=\"");
                snapshot.push_str(escape_xml(key).as_str());
                snapshot.push_str("\">");
                snapshot.push_str(escape_xml(value).as_str());
                snapshot.push_str("</entry>\n");
            }
            snapshot.push_str("  </metadata>\n");
        }
        push_xml_list_element(&mut snapshot, "narrative_context", "entry", blocks, 1);
        push_xml_text_element(&mut snapshot, "working_history", history, 1);
        snapshot.push_str("</system_prompt_injection>");
        snapshot
    }
}

#[async_trait]
impl QianjiMechanism for ContextAnnotator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let narrative_blocks = self.collect_narrative_blocks(context)?;
        let history_seed = self.resolve_history_seed(context);
        let persona_reference = resolve_semantic_reference(&self.persona_id, context)?;

        let persona = Self::resolve_persona_profile(persona_reference.as_str())?;
        let persona_id = persona.id.clone();

        // --- REAL-TIME BATTLE REPORTING ---
        println!(
            "\n\033[1;34m[Node: {}]\033[0m Activating Avatar: \033[1;33m{}\033[0m",
            self.output_key, persona_id
        );
        if self.execution_mode == NodeAnnotationExecutionMode::Appended {
            println!("  > Mode: Appended (Preserving Session Context)");
        }
        // ----------------------------------

        let snapshot = Self::assemble_snapshot(&persona, &narrative_blocks, &history_seed);

        let mut data = serde_json::Map::new();
        data.insert(self.output_key.clone(), json!(snapshot));
        data.insert(self.metadata_key("persona_id"), json!(persona_id));
        data.insert(
            self.metadata_key("execution_mode"),
            json!(self.execution_mode.as_str()),
        );
        data.insert(
            self.metadata_key("input_keys"),
            json!(self.input_keys.clone()),
        );
        if self.execution_mode == NodeAnnotationExecutionMode::Appended {
            data.insert(
                self.metadata_key("history_key"),
                json!(self.history_key.clone()),
            );
        }
        if let Some(template_target) = self.template_target.as_deref() {
            data.insert(
                self.metadata_key("template_target"),
                json!(resolve_semantic_reference(template_target, context)?),
            );
        }
        if let Some(updated_history) =
            self.merge_history_for_appended_mode(&history_seed, &snapshot)
        {
            data.insert(self.history_key.clone(), json!(updated_history));
        }

        Ok(QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        8.0
    }
}

fn persona_profile_from_reference(persona_reference: &str) -> Result<PersonaProfile, String> {
    let id = persona_reference.trim();
    if id.is_empty() {
        return Err("persona reference must not be empty".to_string());
    }
    let name = id
        .split(['-', '_'])
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                let mut out = String::new();
                out.push(first.to_ascii_uppercase());
                out.push_str(chars.as_str());
                out
            })
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(PersonaProfile {
        id: id.to_string(),
        name,
        voice_tone: "Calm, practical, and context-grounded.".to_string(),
        background: None,
        guidelines: vec!["Respond with concise and actionable guidance.".to_string()],
        style_anchors: Vec::new(),
        cot_template:
            "Extract constraints, reason about feasibility, then produce one executable output."
                .to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    })
}

fn push_xml_text_element(snapshot: &mut String, tag: &str, value: &str, indent: usize) {
    let padding = "  ".repeat(indent);
    snapshot.push_str(padding.as_str());
    snapshot.push('<');
    snapshot.push_str(tag);
    snapshot.push('>');
    snapshot.push_str(escape_xml(value).as_str());
    snapshot.push_str("</");
    snapshot.push_str(tag);
    snapshot.push_str(">\n");
}

fn push_xml_list_element(
    snapshot: &mut String,
    tag: &str,
    item_tag: &str,
    values: &[String],
    indent: usize,
) {
    let padding = "  ".repeat(indent);
    snapshot.push_str(padding.as_str());
    snapshot.push('<');
    snapshot.push_str(tag);
    snapshot.push_str(">\n");
    for value in values {
        push_xml_text_element(snapshot, item_tag, value, indent + 1);
    }
    snapshot.push_str(padding.as_str());
    snapshot.push_str("</");
    snapshot.push_str(tag);
    snapshot.push_str(">\n");
}

fn escape_xml(raw: &str) -> String {
    raw.chars().fold(String::new(), |mut escaped, ch| {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
        escaped
    })
}
