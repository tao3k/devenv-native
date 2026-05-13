//! Executors annotation context surface for `xiuxian-qianji`.

use super::persona_markdown::persona_profile_from_markdown;
use crate::contracts::{FlowInstruction, NodeQianhuanExecutionMode, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::{
    context_value_to_text, lookup_context_path, resolve_semantic_content,
    resolve_semantic_reference, resolve_wendao_uri_with_zhenfa,
};
use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::{PersonaProfile, PersonaRegistry};
use xiuxian_wendao_core::WendaoResourceUri;

const MAX_SNAPSHOT_COMPACTION_DEPTH: usize = 4;

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
    /// Reference to the `ThousandFaces` orchestrator.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Reference to the Persona Registry.
    pub registry: Arc<PersonaRegistry>,
    /// Target persona ID defined in the registry.
    pub persona_id: String,
    /// Optional logical template target associated with this node.
    pub template_target: Option<String>,
    /// Context window behavior for this annotation node.
    pub execution_mode: NodeQianhuanExecutionMode,
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
            NodeQianhuanExecutionMode::Isolated => String::new(),
            NodeQianhuanExecutionMode::Appended => context
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
        if self.execution_mode != NodeQianhuanExecutionMode::Appended {
            return None;
        }
        if current_history.is_empty() {
            return Some(snapshot.to_string());
        }
        Some(format!("{current_history}\n{snapshot}"))
    }

    fn resolve_persona_profile(&self, persona_reference: &str) -> Result<PersonaProfile, String> {
        if persona_reference.trim_start().starts_with("wendao://") {
            return self.resolve_wendao_persona_profile(persona_reference);
        }
        self.registry
            .get(persona_reference)
            .ok_or_else(|| format!("Persona '{persona_reference}' not found"))
    }

    fn resolve_wendao_persona_profile(&self, uri: &str) -> Result<PersonaProfile, String> {
        let parsed_uri = WendaoResourceUri::parse(uri)
            .map_err(|error| format!("invalid persona semantic URI '{uri}': {error}"))?;
        let canonical_uri = parsed_uri.canonical_uri();
        let markdown = resolve_wendao_uri_with_zhenfa(canonical_uri.as_str())?;
        let parsed_profile =
            persona_profile_from_markdown(canonical_uri.as_str(), markdown.as_str());
        if let Some(existing) = self.registry.get(parsed_profile.id.as_str()) {
            return Ok(existing);
        }
        Ok(parsed_profile)
    }
}

#[async_trait]
impl QianjiMechanism for ContextAnnotator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let narrative_blocks = self.collect_narrative_blocks(context)?;
        let history_seed = self.resolve_history_seed(context);
        let persona_reference = resolve_semantic_reference(&self.persona_id, context)?;

        let persona = self.resolve_persona_profile(persona_reference.as_str())?;
        let persona_id = persona.id.clone();

        // --- REAL-TIME BATTLE REPORTING ---
        println!(
            "\n\033[1;34m[Node: {}]\033[0m Activating Avatar: \033[1;33m{}\033[0m",
            self.output_key, persona_id
        );
        if self.execution_mode == NodeQianhuanExecutionMode::Appended {
            println!("  > Mode: Appended (Preserving Session Context)");
        }
        // ----------------------------------

        let snapshot = self
            .orchestrator
            .assemble_snapshot(&persona, narrative_blocks, &history_seed)
            .await
            .map_err(|e| format!("Qianhuan annotation failed: {e}"))?;

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
        if self.execution_mode == NodeQianhuanExecutionMode::Appended {
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
