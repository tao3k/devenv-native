//! Artifact-cache helpers for prompt-context packs.

use serde::Serialize;
use xiuxian_db_store::artifact_cache::{
    AgentArtifactKeyParts, ArtifactBlobCache, ArtifactCacheError, ArtifactKey, ArtifactKind,
    ArtifactReadThrough, agent_artifact_key, read_through_artifact_bytes,
};

use crate::{InjectionPolicy, InjectionSnapshot, PromptContextBlock, RoleMixProfile};

const PROMPT_CONTEXT_PACK_SCHEMA: &str = "xiuxian_qianhuan.prompt_context_pack.v1";
const PROMPT_CONTEXT_ARTIFACT_BACKEND: &str = "qianhuan-prompt-context";

/// Deterministic artifact identity for a prompt-context pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptContextPackIdentity {
    /// Digest of the context source scope.
    pub source_digest: String,
    /// Digest of the policy and role profile that shape the pack.
    pub profile_digest: String,
    /// Digest of the retained prompt-context blocks.
    pub shard_digest: String,
}

impl PromptContextPackIdentity {
    /// Derive a stable identity from snapshot content.
    ///
    /// This intentionally excludes the snapshot id and turn id so identical
    /// prompt-context content in the same session can hit the artifact substrate.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when snapshot serialization fails.
    pub fn from_snapshot_content(snapshot: &InjectionSnapshot) -> Result<Self, ArtifactCacheError> {
        Ok(Self {
            source_digest: digest_bytes(snapshot.session_id.as_ref().as_bytes()),
            profile_digest: digest_json(
                "serializing prompt-context profile identity",
                &PromptContextPackProfileDigest {
                    policy: &snapshot.policy,
                    role_mix: &snapshot.role_mix,
                },
            )?,
            shard_digest: digest_json(
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

/// Read-through outcome for a prompt-context pack.
pub struct PromptContextPackReadThrough {
    key: ArtifactKey,
    artifact: ArtifactReadThrough,
}

impl PromptContextPackReadThrough {
    /// Cache key used for this read-through operation.
    #[must_use]
    pub const fn key(&self) -> &ArtifactKey {
        &self.key
    }

    /// Artifact read-through details.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactReadThrough {
        &self.artifact
    }

    /// Borrow the prompt-context pack bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.artifact.bytes()
    }

    /// Whether the bytes came from the cache.
    #[must_use]
    pub const fn cache_hit(&self) -> bool {
        self.artifact.cache_hit()
    }

    /// Number of prompt-context pack bytes returned.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.artifact.byte_len()
    }

    /// Consume this outcome and return the underlying read-through artifact.
    #[must_use]
    pub fn into_artifact(self) -> ArtifactReadThrough {
        self.artifact
    }
}

/// Build the shared agent namespace key for a prompt-context pack.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when any identity component is not safe for
/// content-addressed artifact storage.
pub fn prompt_context_pack_key(
    identity: PromptContextPackIdentity,
) -> Result<ArtifactKey, ArtifactCacheError> {
    agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::PromptContextPack,
        source_digest: identity.source_digest,
        profile_digest: identity.profile_digest,
        shard_digest: identity.shard_digest,
    })
}

/// Serialize an injection snapshot as a versioned prompt-context pack.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when snapshot serialization fails.
pub fn prompt_context_pack_bytes(
    snapshot: &InjectionSnapshot,
) -> Result<Vec<u8>, ArtifactCacheError> {
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

/// Read or build a prompt-context pack through the shared artifact substrate.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when key construction, cache read/write, or
/// prompt-context pack construction fails.
pub fn read_through_prompt_context_pack(
    cache: &dyn ArtifactBlobCache,
    identity: PromptContextPackIdentity,
    build: impl FnOnce() -> Result<Vec<u8>, ArtifactCacheError>,
) -> Result<PromptContextPackReadThrough, ArtifactCacheError> {
    let key = prompt_context_pack_key(identity)?;
    let artifact = read_through_artifact_bytes(cache, &key, build)?;
    Ok(PromptContextPackReadThrough { key, artifact })
}

/// Read or build the versioned pack bytes for an injection snapshot.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when snapshot identity serialization,
/// prompt-context pack serialization, or cache IO fails.
pub fn read_through_injection_snapshot_pack(
    cache: &dyn ArtifactBlobCache,
    snapshot: &InjectionSnapshot,
) -> Result<PromptContextPackReadThrough, ArtifactCacheError> {
    let identity = PromptContextPackIdentity::from_snapshot_content(snapshot)?;
    read_through_prompt_context_pack(cache, identity, || prompt_context_pack_bytes(snapshot))
}

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

#[derive(Serialize)]
struct PromptContextPackProfileDigest<'a> {
    policy: &'a InjectionPolicy,
    role_mix: &'a Option<RoleMixProfile>,
}

#[derive(Serialize)]
struct PromptContextPackShardDigest<'a> {
    blocks: &'a [PromptContextBlock],
    total_chars: usize,
    dropped_block_ids: &'a [String],
    truncated_block_ids: &'a [String],
}

fn digest_json<T: Serialize>(
    action: &'static str,
    value: &T,
) -> Result<String, ArtifactCacheError> {
    let bytes = serde_json::to_vec(value).map_err(|error| artifact_backend_error(action, error))?;
    Ok(digest_bytes(bytes.as_slice()))
}

fn digest_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

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
