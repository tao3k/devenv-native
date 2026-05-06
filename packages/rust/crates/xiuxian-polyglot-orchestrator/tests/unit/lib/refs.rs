use crate::{ContractOwner, PolyglotLane, RouteProfileRef};

#[test]
fn document_extract_ref_keeps_analyzer_ownership() {
    let reference = RouteProfileRef::document_extract("/analysis/document-extract");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Analyzer);
    assert_eq!(reference.route, "/analysis/document-extract");
    assert!(reference.profile.is_none());
}

#[test]
fn ocr_ref_keeps_attachment_contract_ownership() {
    let reference = RouteProfileRef::ocr_shards("/analysis/pdf-ocr-shards", "ocr_shard_v1");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Attachments);
    assert_eq!(reference.schema_version.as_deref(), Some("ocr_shard_v1"));
}

#[test]
fn julia_profile_ref_keeps_julia_contract_ownership() {
    let reference = RouteProfileRef::julia_profile("memory.julia_compute", "episodic_recall", "v1");

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.profile.as_deref(), Some("episodic_recall"));
}
