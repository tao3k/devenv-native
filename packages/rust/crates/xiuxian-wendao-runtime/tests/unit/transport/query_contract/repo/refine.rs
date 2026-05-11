use crate::transport::{RefineDocRequest, validate_refine_doc_request};

#[test]
fn refine_doc_request_validation_accepts_base64_user_hints() {
    assert_eq!(
        validate_refine_doc_request(
            "gateway-sync",
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
            Some("RXhwbGFpbiB0aGlzIGVudHJ5cG9pbnQ="),
        ),
        Ok(RefineDocRequest {
            repo_id: "gateway-sync".to_string(),
            entity_id: "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
            user_hints: Some("Explain this entrypoint".to_string()),
        })
    );
    assert_eq!(
        validate_refine_doc_request(
            "gateway-sync",
            "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
            Some("   "),
        ),
        Ok(RefineDocRequest {
            repo_id: "gateway-sync".to_string(),
            entity_id: "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
            user_hints: None,
        })
    );
}

#[test]
fn refine_doc_request_validation_rejects_blank_repo() {
    assert_eq!(
        validate_refine_doc_request("   ", "repo:gateway-sync:symbol:GatewaySyncPkg.solve", None,),
        Err("refine doc repo must not be blank".to_string())
    );
}

#[test]
fn refine_doc_request_validation_rejects_blank_entity_id() {
    assert_eq!(
        validate_refine_doc_request("gateway-sync", "   ", None),
        Err("refine doc entity_id must not be blank".to_string())
    );
}

#[test]
fn refine_doc_request_validation_rejects_invalid_base64_user_hints() {
    let Err(error) = validate_refine_doc_request(
        "gateway-sync",
        "repo:gateway-sync:symbol:GatewaySyncPkg.solve",
        Some("%%%"),
    ) else {
        panic!("invalid base64 user hints should fail");
    };
    assert!(error.starts_with("refine doc user_hints must be valid Base64:"));
}
