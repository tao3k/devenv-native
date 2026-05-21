//! Episteme audio claim promotion proposal artifact writer.
//!
//! This module writes reviewed audio claims into deterministic proposal
//! artifacts. It does not generate RDF and does not mutate ontology sources.

use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::{
    EpistemeError, read_model::EpistemeAudioReviewedClaimObjectKind,
    read_model::EpistemeAudioReviewedClaimReadModelRequest,
    read_model::materialize_episteme_audio_reviewed_claim_seed, safe_run_id,
};

const AUDIO_CLAIM_PROPOSAL_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_audio_claim_promotion_proposal.v1";
const CLAIMS_TSV: &str = "claims.tsv";
const RECEIPT_JSON: &str = "receipt.json";

/// Request for writing a reviewed audio claim promotion proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemeAudioClaimPromotionProposalRequest {
    /// Safe ASCII proposal id.
    pub proposal_id: String,
    /// Reviewed audio claims and their source evidence.
    pub reviewed_claims: EpistemeAudioReviewedClaimReadModelRequest,
}

impl EpistemeAudioClaimPromotionProposalRequest {
    /// Create a reviewed audio claim promotion proposal request.
    #[must_use]
    pub fn new(
        proposal_id: impl Into<String>,
        reviewed_claims: EpistemeAudioReviewedClaimReadModelRequest,
    ) -> Self {
        Self {
            proposal_id: proposal_id.into(),
            reviewed_claims,
        }
    }
}

/// Report emitted after writing a reviewed audio claim promotion proposal.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeAudioClaimPromotionProposalReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII proposal id.
    pub proposal_id: String,
    /// Concrete proposal directory.
    pub proposal_dir: PathBuf,
    /// Written reviewed-claims TSV path.
    pub claims_path: PathBuf,
    /// Written receipt JSON path.
    pub receipt_path: PathBuf,
    /// Number of reviewed claim rows written.
    pub claim_count: usize,
    /// Number of distinct evidence segment ids referenced by the claims.
    pub evidence_segment_count: usize,
    /// Whether RDF materialization was performed.
    pub rdf_materialization_performed: bool,
    /// Whether ontology source files were written.
    pub ontology_source_write_performed: bool,
    /// Whether direct raw transcript text promotion is allowed.
    pub raw_transcript_promotion_allowed: bool,
}

/// Write deterministic reviewed audio claim promotion proposal artifacts.
///
/// # Errors
///
/// Returns an error when the proposal id is unsafe, reviewed claim validation
/// fails, or proposal artifacts cannot be written.
pub fn write_episteme_audio_claim_promotion_proposal(
    request: &EpistemeAudioClaimPromotionProposalRequest,
    proposal_root: impl AsRef<Path>,
) -> Result<EpistemeAudioClaimPromotionProposalReport, EpistemeError> {
    safe_run_id(&request.proposal_id)?;
    materialize_episteme_audio_reviewed_claim_seed(&request.reviewed_claims)?;

    let proposal_dir = proposal_root.as_ref().join(&request.proposal_id);
    let claims_path = proposal_dir.join(CLAIMS_TSV);
    let receipt_path = proposal_dir.join(RECEIPT_JSON);
    let evidence_segment_count = request
        .reviewed_claims
        .claims
        .iter()
        .map(|claim| claim.evidence_segment_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let report = EpistemeAudioClaimPromotionProposalReport {
        schema_version: AUDIO_CLAIM_PROPOSAL_SCHEMA_VERSION,
        proposal_id: request.proposal_id.clone(),
        proposal_dir,
        claims_path,
        receipt_path,
        claim_count: request.reviewed_claims.claims.len(),
        evidence_segment_count,
        rdf_materialization_performed: false,
        ontology_source_write_performed: false,
        raw_transcript_promotion_allowed: false,
    };

    create_dir_all(&report.proposal_dir)?;
    write_claims_tsv(&report.claims_path, request)?;
    write_receipt_json(&report.receipt_path, &report)?;
    Ok(report)
}

fn create_dir_all(path: &Path) -> Result<(), EpistemeError> {
    fs::create_dir_all(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_claims_tsv(
    path: &Path,
    request: &EpistemeAudioClaimPromotionProposalRequest,
) -> Result<(), EpistemeError> {
    let mut file = fs::File::create(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(
        file,
        "claim_id\tevidence_segment_id\tontology_subject\tontology_predicate\tontology_object\tobject_kind\treviewer_id\treviewed_at\tevidence_quote_sha256\treview_note_sha256\tconfidence\tstatus"
    )
    .map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    for claim in &request.reviewed_claims.claims {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\tpromotion-candidate",
            claim.claim_id,
            claim.evidence_segment_id,
            claim.ontology_subject,
            claim.ontology_predicate,
            claim.ontology_object,
            object_kind_as_str(claim.object_kind),
            claim.reviewer_id,
            claim.reviewed_at,
            claim.evidence_quote_sha256,
            claim.review_note_sha256.as_deref().unwrap_or_default(),
            claim.confidence
        )
        .map_err(|source| EpistemeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn write_receipt_json(
    path: &Path,
    report: &EpistemeAudioClaimPromotionProposalReport,
) -> Result<(), EpistemeError> {
    let raw = serde_json::to_string_pretty(report).map_err(|source| EpistemeError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, format!("{raw}\n")).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

const fn object_kind_as_str(kind: EpistemeAudioReviewedClaimObjectKind) -> &'static str {
    match kind {
        EpistemeAudioReviewedClaimObjectKind::Entity => "entity",
        EpistemeAudioReviewedClaimObjectKind::Literal => "literal",
        EpistemeAudioReviewedClaimObjectKind::Quantity => "quantity",
    }
}
