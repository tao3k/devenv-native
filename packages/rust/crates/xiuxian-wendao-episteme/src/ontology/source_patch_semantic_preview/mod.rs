//! Non-mutating semantic read-model preview from reviewed source-patch rows.

mod api;
mod types;

pub use api::write_episteme_ontology_source_patch_semantic_preview;
pub(crate) use types::{
    EpistemeOntologySemanticEvidenceRow, EpistemeOntologySemanticObjectRow,
    EpistemeOntologySemanticRelationRow,
};
pub use types::{
    EpistemeOntologySemanticProjectionStateRow, EpistemeOntologySourcePatchSemanticPreviewReport,
    EpistemeOntologySourcePatchSemanticPreviewRequest,
};
