//! Multi-layer orchestrator for xiuxian-qianhuan prompt assembly.

use std::fmt::Write as _;
use std::sync::Arc;

use crate::error::{InjectionError, Result};
use crate::persona::PersonaProfile;
use crate::transmuter::ToneTransmuter;
use crate::xml::SYSTEM_PROMPT_INJECTION_TAG;

const MIN_CONTEXT_CONFIDENCE: f64 = 0.65;

/// Logical layers used to compose an injection snapshot.
pub enum InjectionLayer {
    /// L0: immutable safety and governance rules.
    Genesis,
    /// L1: persona tone and reasoning style steering.
    Persona,
    /// L2: transformed narrative/knowledge blocks.
    Narrative,
    /// L3: recency/working-memory context.
    Working,
}

/// Assembles layered prompt snapshots with optional narrative transmutation.
pub struct ThousandFacesOrchestrator {
    genesis_rules: String,
    transmuter: Option<Arc<dyn ToneTransmuter>>,
}

impl ThousandFacesOrchestrator {
    /// Creates a new orchestrator with fixed genesis rules and optional transmuter.
    #[must_use]
    pub fn new(genesis_rules: String, transmuter: Option<Arc<dyn ToneTransmuter>>) -> Self {
        Self {
            genesis_rules,
            transmuter,
        }
    }

    /// Assembles the final XML system prompt snapshot asynchronously.
    ///
    /// Narrative blocks are passed through the configured transmuter when present.
    ///
    /// # Errors
    ///
    /// Returns an error when narrative transmutation fails, XML formatting cannot
    /// be written into the output buffer, or the final XML payload is unbalanced.
    pub async fn assemble_snapshot(
        &self,
        persona: &PersonaProfile,
        narrative_blocks: Vec<String>,
        history: &str,
    ) -> Result<String> {
        Self::enforce_context_confidence(persona, &narrative_blocks)?;
        let mut full_prompt = String::with_capacity(4096);

        self.append_genesis_layer(&mut full_prompt)?;
        Self::append_persona_layer(&mut full_prompt, persona)?;
        self.append_narrative_layer(&mut full_prompt, persona, narrative_blocks)
            .await?;
        Self::append_working_layer(&mut full_prompt, history)?;

        let final_xml = Self::wrap_system_prompt(&full_prompt)?;
        Self::validate_xml(&final_xml)?;

        Ok(final_xml)
    }

    fn append_genesis_layer(&self, full_prompt: &mut String) -> Result<()> {
        let genesis_rules = escape_xml_text(self.genesis_rules.as_str());
        write_xml(
            full_prompt,
            format_args!("<genesis_rules>\n{genesis_rules}\n</genesis_rules>\n"),
        )
    }

    fn append_persona_layer(full_prompt: &mut String, persona: &PersonaProfile) -> Result<()> {
        full_prompt.push_str("<persona_steering>\n");
        append_persona_core(full_prompt, persona)?;
        if let Some(background) = persona.background.as_deref()
            && !background.trim().is_empty()
        {
            write_xml(
                full_prompt,
                format_args!(
                    "  <background>{}</background>\n",
                    escape_xml_text(background)
                ),
            )?;
        }
        append_persona_guidelines(full_prompt, persona)?;
        append_persona_anchors(full_prompt, persona);
        append_persona_forbidden_terms(full_prompt, persona)?;
        full_prompt.push_str("</persona_steering>\n");
        Ok(())
    }

    async fn append_narrative_layer(
        &self,
        full_prompt: &mut String,
        persona: &PersonaProfile,
        narrative_blocks: Vec<String>,
    ) -> Result<()> {
        full_prompt.push_str("<narrative_context>\n");
        match &self.transmuter {
            Some(transmuter) => {
                append_transmuted_narrative_entries(
                    full_prompt,
                    persona,
                    &narrative_blocks,
                    transmuter.as_ref(),
                )
                .await?;
            }
            None => append_raw_narrative_entries(full_prompt, &narrative_blocks)?,
        }
        full_prompt.push_str("</narrative_context>\n");
        Ok(())
    }

    fn append_working_layer(full_prompt: &mut String, history: &str) -> Result<()> {
        write_xml(
            full_prompt,
            format_args!(
                "<working_history>\n{}\n</working_history>\n",
                escape_xml_text(history)
            ),
        )
    }

    fn wrap_system_prompt(full_prompt: &str) -> Result<String> {
        let mut final_xml = String::with_capacity(full_prompt.len() + 64);
        write_xml(
            &mut final_xml,
            format_args!(
                "<{SYSTEM_PROMPT_INJECTION_TAG}>\n{full_prompt}\n</{SYSTEM_PROMPT_INJECTION_TAG}>"
            ),
        )?;
        Ok(final_xml)
    }

    fn validate_xml(xml: &str) -> Result<()> {
        let mut stack = Vec::new();
        let mut cursor = 0;
        while let Some(tag) = next_xml_tag(xml, cursor)? {
            cursor = tag.next_cursor;
            apply_xml_tag(&mut stack, tag.content)?;
        }

        if let Some(open_tag) = stack.pop() {
            return Err(InjectionError::XmlValidationError(format!(
                "Unclosed tag at end of input: <{open_tag}>"
            )));
        }

        Ok(())
    }

    fn enforce_context_confidence(
        persona: &PersonaProfile,
        narrative_blocks: &[String],
    ) -> Result<()> {
        if persona.style_anchors.is_empty() {
            return Ok(());
        }

        let evidence = narrative_blocks
            .iter()
            .map(|block| block.to_lowercase())
            .collect::<Vec<_>>();
        let missing = persona
            .style_anchors
            .iter()
            .filter(|anchor| {
                let anchor_lower = anchor.to_lowercase();
                !evidence
                    .iter()
                    .any(|block| block.contains(anchor_lower.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>();
        let total = u32::try_from(persona.style_anchors.len()).map_or(f64::INFINITY, f64::from);
        let missing_count = u32::try_from(missing.len()).map_or(f64::INFINITY, f64::from);
        let matched = total - missing_count;

        let ccs = if total == 0.0 {
            1.0
        } else {
            (matched / total).clamp(0.0, 1.0)
        };
        if ccs < MIN_CONTEXT_CONFIDENCE {
            return Err(InjectionError::ContextInsufficient {
                ccs,
                missing_info: missing.join(", "),
            });
        }
        Ok(())
    }
}

fn append_persona_core(full_prompt: &mut String, persona: &PersonaProfile) -> Result<()> {
    write_xml(
        full_prompt,
        format_args!("  <tone>{}</tone>\n", escape_xml_text(&persona.voice_tone)),
    )?;
    write_xml(
        full_prompt,
        format_args!(
            "  <thought_pattern>{}</thought_pattern>\n",
            escape_xml_text(&persona.cot_template)
        ),
    )
}

fn append_persona_guidelines(full_prompt: &mut String, persona: &PersonaProfile) -> Result<()> {
    if !persona.guidelines.is_empty() {
        full_prompt.push_str("  <guidelines>\n");
        for guideline in &persona.guidelines {
            write_xml(
                full_prompt,
                format_args!("    <rule>{}</rule>\n", escape_xml_text(guideline)),
            )?;
        }
        full_prompt.push_str("  </guidelines>\n");
    }
    Ok(())
}

fn append_persona_anchors(full_prompt: &mut String, persona: &PersonaProfile) {
    full_prompt.push_str("  <anchors>");
    full_prompt.push_str(
        &persona
            .style_anchors
            .iter()
            .map(|anchor| escape_xml_text(anchor))
            .collect::<Vec<_>>()
            .join(", "),
    );
    full_prompt.push_str("</anchors>\n");
}

fn append_persona_forbidden_terms(
    full_prompt: &mut String,
    persona: &PersonaProfile,
) -> Result<()> {
    if !persona.forbidden_words.is_empty() {
        full_prompt.push_str("  <forbidden_terms>\n");
        for term in &persona.forbidden_words {
            write_xml(
                full_prompt,
                format_args!("    <term>{}</term>\n", escape_xml_text(term)),
            )?;
        }
        full_prompt.push_str("  </forbidden_terms>\n");
    }
    Ok(())
}

fn append_narrative_entry(full_prompt: &mut String, text: &str) -> Result<()> {
    write_xml(
        full_prompt,
        format_args!("  <entry>{}</entry>\n", escape_xml_text(text)),
    )
}

fn append_raw_narrative_entries(
    full_prompt: &mut String,
    narrative_blocks: &[String],
) -> Result<()> {
    narrative_blocks
        .iter()
        .try_for_each(|block| append_narrative_entry(full_prompt, block))
}

async fn append_transmuted_narrative_entries(
    full_prompt: &mut String,
    persona: &PersonaProfile,
    narrative_blocks: &[String],
    transmuter: &dyn ToneTransmuter,
) -> Result<()> {
    for block in narrative_blocks {
        let shifted = transmuter.transmute(block, persona).await?;
        append_narrative_entry(full_prompt, &shifted)?;
    }
    Ok(())
}

struct XmlTag<'a> {
    content: &'a str,
    next_cursor: usize,
}

fn next_xml_tag(xml: &str, cursor: usize) -> Result<Option<XmlTag<'_>>> {
    let Some(relative_start) = xml[cursor..].find('<') else {
        return Ok(None);
    };
    let start = cursor + relative_start + 1;
    let Some(relative_end) = xml[start..].find('>') else {
        return Err(InjectionError::XmlValidationError(
            "Unclosed tag".to_string(),
        ));
    };
    let end = start + relative_end;
    Ok(Some(XmlTag {
        content: &xml[start..end],
        next_cursor: end + 1,
    }))
}

fn apply_xml_tag<'a>(stack: &mut Vec<&'a str>, tag_content: &'a str) -> Result<()> {
    if let Some(tag_name) = tag_content.strip_prefix('/') {
        return close_xml_tag(stack, tag_name);
    }
    if !tag_content.ends_with('/')
        && let Some(tag_name) = tag_content.split_whitespace().next()
        && !tag_name.is_empty()
    {
        stack.push(tag_name);
    }
    Ok(())
}

fn close_xml_tag(stack: &mut Vec<&str>, tag_name: &str) -> Result<()> {
    match stack.pop() {
        Some(open_tag) if open_tag == tag_name => Ok(()),
        Some(open_tag) => Err(InjectionError::XmlValidationError(format!(
            "Mismatched tag: expected </{open_tag}>, found </{tag_name}>"
        ))),
        None => Err(InjectionError::XmlValidationError(format!(
            "Unexpected closing tag: </{tag_name}>"
        ))),
    }
}

fn write_xml(buffer: &mut String, args: std::fmt::Arguments<'_>) -> Result<()> {
    buffer.write_fmt(args).map_err(|error| {
        InjectionError::Internal(format!("failed to format XML snapshot: {error}"))
    })
}

fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
