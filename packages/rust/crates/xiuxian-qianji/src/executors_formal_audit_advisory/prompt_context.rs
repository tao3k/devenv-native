//! Qianji-local prompt-context contracts for advisory audit planning.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "advisory-prompt-pack-cache")]
use xiuxian_db_store::artifact_cache::{
    AgentArtifactKeyParts, ArtifactBlobCache, ArtifactCacheError, ArtifactKey, ArtifactKind,
    ArtifactReadThrough, agent_artifact_key, fetch_through_artifact_bytes,
};

#[cfg(feature = "advisory-prompt-pack-cache")]
const PROMPT_CONTEXT_PACK_SCHEMA: &str = "xiuxian_qianji.prompt_context_pack.v1";
#[cfg(feature = "advisory-prompt-pack-cache")]
const PROMPT_CONTEXT_ARTIFACT_BACKEND: &str = "qianji-prompt-context";

/// Profile defining an advisory role's voice, constraints, and reasoning style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaProfile {
    /// Unique role identifier.
    pub id: String,
    /// Friendly display name.
    pub name: String,
    /// Detailed description of the voice and tone.
    pub voice_tone: String,
    /// Detailed background or system instructions for this role.
    #[serde(default)]
    pub background: Option<String>,
    /// Explicit role guidelines.
    #[serde(default)]
    pub guidelines: Vec<String>,
    /// Keywords or anchors that must be present in grounding context.
    pub style_anchors: Vec<String>,
    /// Template used for reasoning.
    pub cot_template: String,
    /// Phrases the role is forbidden to use.
    pub forbidden_words: Vec<String>,
    /// Optional role metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// In-memory registry for advisory personas.
#[derive(Debug, Clone, Default)]
pub(super) struct PersonaRegistry {
    personas: HashMap<String, PersonaProfile>,
}

impl PersonaRegistry {
    /// Creates a registry with Qianji-owned built-in advisory personas.
    #[must_use]
    pub(super) fn with_builtins() -> Self {
        let mut registry = Self::default();
        registry.register(PersonaProfile {
            id: "strict_teacher".to_string(),
            name: "Strict Teacher".to_string(),
            voice_tone: "Precise, critical, and evidence-driven.".to_string(),
            background: Some(
                "Reviews contract findings for ambiguity, missing evidence, and unsafe assumptions."
                    .to_string(),
            ),
            guidelines: vec![
                "Prefer deterministic evidence over speculative critique.".to_string(),
                "Name the blocking contract risk before recommending remediation.".to_string(),
            ],
            style_anchors: vec!["contract".to_string(), "evidence".to_string()],
            cot_template:
                "Identify the contract claim, verify the evidence, then produce a bounded finding."
                    .to_string(),
            forbidden_words: Vec::new(),
            metadata: HashMap::new(),
        });
        registry.register(PersonaProfile {
            id: "artisan-engineer".to_string(),
            name: "Artisan Engineer".to_string(),
            voice_tone: "Practical, implementation-aware, and concise.".to_string(),
            background: Some(
                "Connects contract feedback to concrete implementation and validation steps."
                    .to_string(),
            ),
            guidelines: vec![
                "Translate findings into directly executable engineering changes.".to_string(),
                "Keep remediation scoped to the failing contract.".to_string(),
            ],
            style_anchors: vec!["implementation".to_string(), "validation".to_string()],
            cot_template:
                "Map the failure to owner code, select the smallest fix, then define validation."
                    .to_string(),
            forbidden_words: Vec::new(),
            metadata: HashMap::new(),
        });
        registry
    }

    pub(super) fn register(&mut self, profile: PersonaProfile) {
        self.personas.insert(profile.id.clone(), profile);
    }

    pub(super) fn get(&self, id: impl AsRef<str>) -> Option<PersonaProfile> {
        self.personas.get(id.as_ref()).cloned()
    }
}

/// Stable block identifier for prompt context replay/audit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct PromptContextBlockId(String);

impl PromptContextBlockId {
    /// Creates a prompt context block identifier.
    #[must_use]
    pub(super) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for PromptContextBlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Scope identifier that binds a block to a session or channel.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct PromptSessionScope(String);

impl PromptSessionScope {
    /// Creates a prompt session scope.
    #[must_use]
    pub(super) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for PromptSessionScope {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Source domain that produced a context block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PromptContextSource {
    /// Retrieved from durable knowledge.
    Knowledge,
    /// Runtime-generated execution hints.
    RuntimeHint,
    /// Governance/policy directives.
    Policy,
}

/// Category used by policy-level budget and ordering rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PromptContextCategory {
    /// Governance/policy guidance.
    Policy,
    /// Durable knowledge content.
    Knowledge,
    /// Runtime hint content.
    RuntimeHint,
}

/// Immutable context block in a typed injection snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct PromptContextBlock {
    /// Stable identifier for audit/replay.
    pub(super) block_id: PromptContextBlockId,
    /// Producer source.
    pub(super) source: PromptContextSource,
    /// Policy category.
    pub(super) category: PromptContextCategory,
    /// Higher value means higher priority.
    pub(super) priority: u16,
    /// Scope identifier, usually a session key.
    pub(super) session_scope: PromptSessionScope,
    /// Rendered payload text/XML.
    pub(super) payload: String,
    /// Character count of payload at snapshot time.
    pub(super) payload_chars: usize,
    /// Whether this block is non-evictable.
    pub(super) anchor: bool,
}

/// Named request for constructing a prompt context block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PromptContextBlockInput {
    /// Stable identifier for audit/replay.
    pub(super) block_id: PromptContextBlockId,
    /// Producer source.
    pub(super) source: PromptContextSource,
    /// Policy category.
    pub(super) category: PromptContextCategory,
    /// Higher value means higher priority.
    pub(super) priority: u16,
    /// Scope identifier, usually a session key.
    pub(super) session_scope: PromptSessionScope,
    /// Rendered payload text/XML.
    pub(super) payload: String,
    /// Whether this block is non-evictable.
    pub(super) anchor: bool,
}

impl PromptContextBlock {
    /// Construct a block and compute `payload_chars` from payload text.
    #[must_use]
    pub(super) fn new(input: PromptContextBlockInput) -> Self {
        let payload = input.payload;
        Self {
            block_id: input.block_id,
            source: input.source,
            category: input.category,
            priority: input.priority,
            session_scope: input.session_scope,
            payload_chars: payload.chars().count(),
            payload,
            anchor: input.anchor,
        }
    }
}

/// Deterministic ordering strategy for snapshot assembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum InjectionOrderStrategy {
    /// Group by category, then descending priority.
    CategoryThenPriority,
}

/// Policy that constrains and orders injection blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectionPolicy {
    /// Maximum number of blocks to keep.
    pub max_blocks: usize,
    /// Maximum char budget across all blocks.
    pub max_chars: usize,
    /// Deterministic ordering strategy.
    pub(super) ordering: InjectionOrderStrategy,
    /// Allowed categories for this turn.
    pub(super) enabled_categories: Vec<PromptContextCategory>,
    /// Non-evictable categories regardless of normal pruning.
    pub(super) anchor_categories: Vec<PromptContextCategory>,
}

impl Default for InjectionPolicy {
    fn default() -> Self {
        Self {
            max_blocks: 12,
            max_chars: 8_000,
            ordering: InjectionOrderStrategy::CategoryThenPriority,
            enabled_categories: vec![
                PromptContextCategory::Policy,
                PromptContextCategory::Knowledge,
                PromptContextCategory::RuntimeHint,
            ],
            anchor_categories: vec![PromptContextCategory::Policy],
        }
    }
}

/// Weighted role item in a role-mix profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleMixRole {
    /// Stable role identifier.
    pub role: String,
    /// Relative role weight.
    pub weight: f32,
}

/// Role-mix profile carried by an injection snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleMixProfile {
    /// Profile identifier for observability/replay.
    pub profile_id: String,
    /// Ordered role list used for this turn.
    pub roles: Vec<RoleMixRole>,
    /// Why this profile was selected.
    pub rationale: String,
}

/// Stable snapshot identifier for injection replay/audit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InjectionSnapshotId(String);

impl InjectionSnapshotId {
    /// Creates an injection snapshot identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for InjectionSnapshotId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Session identifier associated with an injection snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct InjectionSessionId(String);

impl InjectionSessionId {
    /// Creates an injection session identifier.
    #[must_use]
    pub(super) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for InjectionSessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Turn sequence identifier associated with an injection snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct InjectionTurnId(u64);

impl InjectionTurnId {
    /// Creates an injection turn identifier.
    #[must_use]
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Immutable turn-level injection snapshot consumed by execution runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InjectionSnapshot {
    /// Snapshot identifier for replay/audit.
    pub snapshot_id: InjectionSnapshotId,
    /// Session identifier.
    pub(super) session_id: InjectionSessionId,
    /// Turn sequence number in this session.
    pub(super) turn_id: InjectionTurnId,
    /// Policy used to produce this snapshot.
    pub policy: InjectionPolicy,
    /// Selected role-mix profile, if any.
    pub role_mix: Option<RoleMixProfile>,
    /// Retained blocks in final snapshot.
    pub(super) blocks: Vec<PromptContextBlock>,
    /// Aggregate chars across retained blocks.
    pub total_chars: usize,
    /// Block IDs dropped by budget policy.
    pub(super) dropped_block_ids: Vec<String>,
    /// Block IDs truncated by budget policy.
    pub(super) truncated_block_ids: Vec<String>,
}

/// Named request for building an injection snapshot.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct InjectionSnapshotInput {
    /// Snapshot identifier for replay/audit.
    pub(super) snapshot_id: InjectionSnapshotId,
    /// Session identifier.
    pub(super) session_id: InjectionSessionId,
    /// Turn sequence number in this session.
    pub(super) turn_id: InjectionTurnId,
    /// Policy used to produce this snapshot.
    pub(super) policy: InjectionPolicy,
    /// Selected role-mix profile, if any.
    pub(super) role_mix: Option<RoleMixProfile>,
    /// Retained blocks in final snapshot.
    pub(super) blocks: Vec<PromptContextBlock>,
}

impl InjectionSnapshot {
    /// Build a snapshot and compute `total_chars` from blocks.
    #[must_use]
    pub(super) fn from_blocks(input: InjectionSnapshotInput) -> Self {
        let total_chars = input.blocks.iter().map(|block| block.payload_chars).sum();
        Self {
            snapshot_id: input.snapshot_id,
            session_id: input.session_id,
            turn_id: input.turn_id,
            policy: input.policy,
            role_mix: input.role_mix,
            blocks: input.blocks,
            total_chars,
            dropped_block_ids: Vec::new(),
            truncated_block_ids: Vec::new(),
        }
    }

    /// Validate key contract invariants for this snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when `total_chars` does not match retained blocks, or
    /// when the snapshot exceeds configured block or character budgets.
    pub fn validate(&self) -> Result<(), String> {
        let computed_chars: usize = self.blocks.iter().map(|block| block.payload_chars).sum();
        if computed_chars != self.total_chars {
            return Err(format!(
                "injection snapshot total_chars mismatch: computed={computed_chars} stored={}",
                self.total_chars
            ));
        }
        if self.blocks.len() > self.policy.max_blocks {
            return Err(format!(
                "injection snapshot exceeds max_blocks: blocks={} max_blocks={}",
                self.blocks.len(),
                self.policy.max_blocks
            ));
        }
        if self.total_chars > self.policy.max_chars {
            return Err(format!(
                "injection snapshot exceeds max_chars: total_chars={} max_chars={}",
                self.total_chars, self.policy.max_chars
            ));
        }
        Ok(())
    }
}

pub(super) fn render_injection_prompt(
    persona: &PersonaProfile,
    blocks: &[PromptContextBlock],
    history: &str,
) -> String {
    let mut rendered = String::from("<system_prompt_injection>\n");
    push_xml_text_element(&mut rendered, "persona_id", persona.id.as_str(), 1);
    push_xml_text_element(&mut rendered, "persona_name", persona.name.as_str(), 1);
    push_xml_text_element(&mut rendered, "voice_tone", persona.voice_tone.as_str(), 1);
    if let Some(background) = persona.background.as_deref() {
        push_xml_text_element(&mut rendered, "background", background, 1);
    }
    push_xml_list_element(&mut rendered, "guidelines", "item", &persona.guidelines, 1);
    push_xml_list_element(
        &mut rendered,
        "style_anchors",
        "item",
        &persona.style_anchors,
        1,
    );
    push_xml_text_element(
        &mut rendered,
        "cot_template",
        persona.cot_template.as_str(),
        1,
    );
    push_xml_list_element(
        &mut rendered,
        "forbidden_words",
        "item",
        &persona.forbidden_words,
        1,
    );
    rendered.push_str("  <narrative_context>\n");
    for block in blocks {
        push_xml_text_element(&mut rendered, "entry", block.payload.as_str(), 2);
    }
    rendered.push_str("  </narrative_context>\n");
    push_xml_text_element(&mut rendered, "working_history", history, 1);
    rendered.push_str("</system_prompt_injection>");
    rendered
}

#[cfg(feature = "advisory-prompt-pack-cache")]
pub(super) struct PromptContextPackReadThrough {
    artifact: ArtifactReadThrough,
}

#[cfg(feature = "advisory-prompt-pack-cache")]
impl PromptContextPackReadThrough {
    pub(super) const fn cache_hit(&self) -> bool {
        self.artifact.cache_hit()
    }

    pub(super) fn byte_len(&self) -> usize {
        self.artifact.byte_len()
    }
}

#[cfg(feature = "advisory-prompt-pack-cache")]
pub(super) fn fetch_through_injection_snapshot_pack(
    cache: &dyn ArtifactBlobCache,
    snapshot: InjectionSnapshot,
) -> Result<PromptContextPackReadThrough, ArtifactCacheError> {
    let identity = PromptContextPackIdentity::from_snapshot_content(&snapshot)?;
    let key = prompt_context_pack_key(identity)?;
    let artifact =
        fetch_through_artifact_bytes(cache, &key, move || prompt_context_pack_bytes(&snapshot))?;
    Ok(PromptContextPackReadThrough { artifact })
}

#[cfg(feature = "advisory-prompt-pack-cache")]
struct PromptContextPackIdentity {
    source: String,
    profile: String,
    shard: String,
}

#[cfg(feature = "advisory-prompt-pack-cache")]
impl PromptContextPackIdentity {
    fn from_snapshot_content(snapshot: &InjectionSnapshot) -> Result<Self, ArtifactCacheError> {
        Ok(Self {
            source: digest_bytes(snapshot.session_id.as_ref().as_bytes()),
            profile: digest_json(
                "serializing prompt-context profile identity",
                &PromptContextPackProfileDigest {
                    policy: &snapshot.policy,
                    role_mix: &snapshot.role_mix,
                },
            )?,
            shard: digest_json(
                "serializing prompt-context shard identity",
                &PromptContextPackShardDigest {
                    blocks: &snapshot.blocks,
                    total_chars: snapshot.total_chars,
                    dropped_block_ids: &snapshot.dropped_block_ids,
                    truncated_block_ids: &snapshot.truncated_block_ids,
                },
            )?,
        })
    }
}

#[cfg(feature = "advisory-prompt-pack-cache")]
fn prompt_context_pack_key(
    identity: PromptContextPackIdentity,
) -> Result<ArtifactKey, ArtifactCacheError> {
    agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::PromptContextPack,
        source_digest: identity.source,
        profile_digest: identity.profile,
        shard_digest: identity.shard,
    })
}

#[cfg(feature = "advisory-prompt-pack-cache")]
fn prompt_context_pack_bytes(snapshot: &InjectionSnapshot) -> Result<Vec<u8>, ArtifactCacheError> {
    serde_json::to_vec(&PromptContextPackEnvelope {
        schema: PROMPT_CONTEXT_PACK_SCHEMA,
        session_id: snapshot.session_id.as_ref(),
        policy: &snapshot.policy,
        role_mix: &snapshot.role_mix,
        blocks: &snapshot.blocks,
        total_chars: snapshot.total_chars,
        dropped_block_ids: &snapshot.dropped_block_ids,
        truncated_block_ids: &snapshot.truncated_block_ids,
    })
    .map_err(|error| artifact_backend_error("serializing prompt-context pack", error))
}

#[cfg(feature = "advisory-prompt-pack-cache")]
#[derive(Serialize)]
struct PromptContextPackEnvelope<'a> {
    schema: &'static str,
    session_id: &'a str,
    policy: &'a InjectionPolicy,
    role_mix: &'a Option<RoleMixProfile>,
    blocks: &'a [PromptContextBlock],
    total_chars: usize,
    dropped_block_ids: &'a [String],
    truncated_block_ids: &'a [String],
}

#[cfg(feature = "advisory-prompt-pack-cache")]
#[derive(Serialize)]
struct PromptContextPackProfileDigest<'a> {
    policy: &'a InjectionPolicy,
    role_mix: &'a Option<RoleMixProfile>,
}

#[cfg(feature = "advisory-prompt-pack-cache")]
#[derive(Serialize)]
struct PromptContextPackShardDigest<'a> {
    blocks: &'a [PromptContextBlock],
    total_chars: usize,
    dropped_block_ids: &'a [String],
    truncated_block_ids: &'a [String],
}

#[cfg(feature = "advisory-prompt-pack-cache")]
fn digest_json<T: Serialize>(
    action: &'static str,
    value: &T,
) -> Result<String, ArtifactCacheError> {
    let bytes = serde_json::to_vec(value).map_err(|error| artifact_backend_error(action, error))?;
    Ok(digest_bytes(bytes.as_slice()))
}

#[cfg(feature = "advisory-prompt-pack-cache")]
fn digest_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(feature = "advisory-prompt-pack-cache")]
fn artifact_backend_error(
    action: &'static str,
    error: impl std::fmt::Display,
) -> ArtifactCacheError {
    ArtifactCacheError::Backend {
        backend: PROMPT_CONTEXT_ARTIFACT_BACKEND,
        action,
        message: error.to_string(),
    }
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
