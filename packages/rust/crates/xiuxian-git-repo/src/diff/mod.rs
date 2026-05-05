//! Repository revision diff interface.

mod summary;

pub use summary::{
    RevisionChangeKind, RevisionDiffSummary, RevisionPathChange, diff_checkout_revisions,
    read_checkout_file_bytes_at_revision,
};
