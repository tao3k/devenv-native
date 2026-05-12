//! `search::manifest` owns Wendao search manifest behavior.

mod fingerprint;
mod input;
mod keyspace;
mod records;
#[cfg(test)]
#[path = "../../../tests/unit/search/manifest/mod.rs"]
mod tests;

pub use fingerprint::SearchFileFingerprint;
pub(crate) use input::SearchRepoPublicationInput;
pub use keyspace::SearchManifestKeyspace;
#[cfg(test)]
pub(crate) use records::build_repo_publication_epoch;
pub use records::{
    SearchManifestRecord, SearchPublicationStorageFormat, SearchRepoCorpusRecord,
    SearchRepoCorpusSnapshotRecord, SearchRepoPublicationRecord, SearchRepoRuntimeRecord,
};
