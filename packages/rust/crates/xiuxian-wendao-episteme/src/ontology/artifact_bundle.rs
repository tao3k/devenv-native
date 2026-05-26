//! Artifact-cache bundle helpers for Episteme ontology run directories.

use std::path::{Path, PathBuf};

use anyhow::Result;
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactKey, ArtifactKind, ArtifactReadThrough, OntologyArtifactKeyParts,
    fetch_through_artifact_bytes, ontology_artifact_key, pack_artifact_directory,
    unpack_artifact_directory,
};

/// Episteme ontology artifact bundle category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemeOntologyArtifactBundleKind {
    /// Source-contract registry snapshot bytes.
    RegistrySnapshot,
    /// Review-gated candidate packet or ledger bytes.
    CandidatePacket,
    /// Candidate Arrow or Parquet read-model bytes.
    CandidateReadModel,
    /// RDF draft bytes produced for review.
    RdfDraft,
    /// Promotion review packet bytes.
    PromotionReviewPacket,
    /// Structural facts, reasoning packet, ledger seed, fill plan, or schedule-plan bytes.
    ReasoningProjection,
}

impl EpistemeOntologyArtifactBundleKind {
    fn artifact_kind(self) -> ArtifactKind {
        match self {
            Self::RegistrySnapshot => ArtifactKind::OntologyRegistrySnapshot,
            Self::CandidatePacket => ArtifactKind::OntologyCandidatePacket,
            Self::CandidateReadModel => ArtifactKind::OntologyCandidateReadModel,
            Self::RdfDraft => ArtifactKind::OntologyRdfDraft,
            Self::PromotionReviewPacket => ArtifactKind::OntologyPromotionReviewPacket,
            Self::ReasoningProjection => ArtifactKind::OntologyReasoningProjection,
        }
    }
}

/// Stable identity for an Episteme ontology run artifact bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyArtifactBundleIdentity {
    /// Bundle category.
    pub kind: EpistemeOntologyArtifactBundleKind,
    /// Source contract, registry, or corpus digest component.
    pub source_digest: String,
    /// Compiler, validation, or ontology profile digest component.
    pub profile_digest: String,
    /// Run, packet, projection, or shard digest component.
    pub run_digest: String,
}

impl EpistemeOntologyArtifactBundleIdentity {
    /// Create an ontology artifact key for this identity.
    ///
    /// # Errors
    ///
    /// Returns an error when any identity component is not safe for artifact
    /// storage.
    pub fn artifact_key(&self) -> Result<ArtifactKey> {
        ontology_artifact_key(OntologyArtifactKeyParts {
            kind: self.kind.artifact_kind(),
            source_digest: self.source_digest.clone(),
            profile_digest: self.profile_digest.clone(),
            shard_digest: self.run_digest.clone(),
        })
        .map_err(Into::into)
    }
}

/// Report emitted after writing an ontology run directory to the artifact cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyArtifactBundleWriteReport {
    /// Artifact key used for the bundle.
    pub artifact_key: ArtifactKey,
    /// Run directory that was packed.
    pub source_dir: PathBuf,
    /// Number of bytes written to the cache backend.
    pub byte_len: usize,
    /// Whether an existing cached bundle was replaced.
    pub replaced: bool,
}

/// Report emitted after restoring an ontology run directory from the artifact cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemeOntologyArtifactBundleRestoreReport {
    /// Artifact key used for the bundle.
    pub artifact_key: ArtifactKey,
    /// Directory receiving restored files.
    pub target_dir: PathBuf,
    /// Number of bytes read from the cache backend.
    pub byte_len: usize,
}

/// Ensure one ontology run directory bundle exists in the shared artifact cache.
///
/// Cache backends with native fetch-through support may coalesce concurrent
/// same-key misses. On a cache hit, this function returns the cached byte count
/// without repacking the source directory.
///
/// # Errors
///
/// Returns an error when the identity is invalid, the run directory cannot be
/// packed on a miss, or the artifact cache backend cannot store the bundle
/// bytes.
pub fn write_episteme_ontology_artifact_bundle(
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    identity: &EpistemeOntologyArtifactBundleIdentity,
    source_dir: impl AsRef<Path>,
) -> Result<EpistemeOntologyArtifactBundleWriteReport> {
    let artifact_key = identity.artifact_key()?;
    let source_dir = source_dir.as_ref().to_path_buf();
    let bundle_dir = source_dir.clone();
    let artifact = fetch_through_artifact_bytes(cache, &artifact_key, move || {
        pack_artifact_directory(bundle_dir.as_path())
    })?;
    Ok(write_report(artifact_key, source_dir, &artifact))
}

/// Restore one ontology run directory from the shared artifact cache.
///
/// # Errors
///
/// Returns an error when the identity is invalid, the artifact cache backend
/// cannot read the bundle, or the bundle cannot be unpacked into the target
/// directory.
pub fn restore_episteme_ontology_artifact_bundle(
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    identity: &EpistemeOntologyArtifactBundleIdentity,
    target_dir: impl AsRef<Path>,
) -> Result<Option<EpistemeOntologyArtifactBundleRestoreReport>> {
    let artifact_key = identity.artifact_key()?;
    let Some(read) = cache.read(&artifact_key)? else {
        return Ok(None);
    };
    let target_dir = target_dir.as_ref();
    unpack_artifact_directory(read.bytes(), target_dir)?;
    Ok(Some(EpistemeOntologyArtifactBundleRestoreReport {
        artifact_key,
        target_dir: target_dir.to_path_buf(),
        byte_len: read.byte_len(),
    }))
}

fn write_report(
    artifact_key: ArtifactKey,
    source_dir: PathBuf,
    artifact: &ArtifactReadThrough,
) -> EpistemeOntologyArtifactBundleWriteReport {
    EpistemeOntologyArtifactBundleWriteReport {
        artifact_key,
        source_dir,
        byte_len: artifact.byte_len(),
        replaced: artifact
            .write_outcome()
            .is_some_and(|outcome| outcome.replaced()),
    }
}
