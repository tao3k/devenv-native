//! Explicit hash-gated source-patch application.

mod engine;
mod preview;
mod types;

pub use engine::apply_episteme_ontology_source_patch;
pub use preview::{
    EpistemeOntologySourcePatchApplyPreviewReport, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyPreviewTarget,
    write_episteme_ontology_source_patch_apply_preview,
};
pub use types::{
    EpistemeOntologySourcePatchAppliedTarget, EpistemeOntologySourcePatchApplyReport,
    EpistemeOntologySourcePatchApplyRequest,
};
