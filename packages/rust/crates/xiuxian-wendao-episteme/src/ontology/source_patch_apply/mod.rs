//! Explicit hash-gated source-patch application.

mod engine;
mod preview;

pub use engine::{
    EpistemeOntologySourcePatchAppliedTarget, EpistemeOntologySourcePatchApplyReport,
    EpistemeOntologySourcePatchApplyRequest, apply_episteme_ontology_source_patch,
};
pub use preview::{
    EpistemeOntologySourcePatchApplyPreviewReport, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyPreviewTarget,
    write_episteme_ontology_source_patch_apply_preview,
};

pub(super) use engine::{
    BEGIN_BLOCK, END_BLOCK, WDSP_NS, reviewed_source_patch_artifacts, sha256_bytes,
};
