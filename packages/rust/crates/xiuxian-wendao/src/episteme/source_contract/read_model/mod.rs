//! Episteme source-contract read-model facade.

mod facade;

#[cfg(feature = "julia")]
pub use facade::build_episteme_wendaograph_quality_request_batches;
pub use facade::{
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimObjectKind,
    EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow,
    EpistemeReadModelMaterialization, EpistemeReadModelRequest, EpistemeReadModelTable,
    admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed,
    materialize_episteme_audio_evidence_review_seed,
    materialize_episteme_audio_reviewed_claim_seed,
    materialize_episteme_ontology_registry_snapshot_read_model_seed,
    materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    materialize_episteme_registry_reference_graph_read_model_seed,
    validate_episteme_read_model_relation_endpoints,
};
