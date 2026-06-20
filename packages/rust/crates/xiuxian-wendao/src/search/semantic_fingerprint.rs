use serde::Serialize;

use crate::search::contracts::{AttachmentSearchHit, ReferenceSearchHit, SourceSymbolHit};

#[must_use]
pub(crate) fn source_symbol_hits_fingerprint(hits: &[SourceSymbolHit]) -> String {
    stable_payload_fingerprint("source_symbol_hits", hits)
}

#[must_use]
pub(crate) fn attachment_hits_fingerprint(hits: &[AttachmentSearchHit]) -> String {
    stable_payload_fingerprint("attachment_hits", hits)
}

#[must_use]
pub(crate) fn reference_hits_fingerprint(hits: &[ReferenceSearchHit]) -> String {
    stable_payload_fingerprint("reference_hits", hits)
}

#[must_use]
pub(crate) fn stable_payload_fingerprint<T: Serialize + ?Sized>(kind: &str, value: &T) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_else(|error| {
        panic!("semantic fingerprint payload should serialize: {error}");
    });
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(&payload);
    hasher.finalize().to_hex().to_string()
}
