use super::support::{fixture_index, semantic_overlay_edge};
use crate::link_graph::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestOptions,
    build_wendao_graph_evidence_request_bundle_with_options,
};

#[test]
fn request_bundle_rejects_conflicting_semantic_inputs() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_neighbor("docs/alpha", "docs/beta", 1, 2, 1, 0.25)
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2));
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("conflicting semantic input variants should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::ConflictingSemanticEvidence
    ));
}

#[test]
fn request_bundle_rejects_seed_outside_projected_nodes() {
    let index = fixture_index();
    let options =
        WendaoGraphEvidenceRequestOptions::default().with_seed("docs/missing#anchor", 1.0);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown seed node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSeedNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_seed_weight() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_seed("docs/alpha", -1.0);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("negative seed weight should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSeedWeight { .. }
    ));
}

#[test]
fn request_bundle_rejects_semantic_neighbor_outside_projected_nodes() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/missing",
        1,
        2,
        1,
        0.25,
    );
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown semantic neighbor node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSemanticNeighborNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_semantic_neighbor_distance() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/beta",
        1,
        2,
        1,
        f64::NAN,
    );
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("non-finite semantic neighbor distance should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSemanticNeighbor {
            field: "distance",
            ..
        }
    ));
}

#[test]
fn request_bundle_rejects_semantic_overlay_outside_projected_nodes() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/missing", 1, 2));
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("unknown semantic overlay node should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::UnknownSemanticOverlayNode { .. }
    ));
}

#[test]
fn request_bundle_rejects_invalid_semantic_overlay_weight() {
    let index = fixture_index();
    let mut edge = semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2);
    edge.weight = f64::INFINITY;
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_overlay_edge(edge);
    let Err(error) = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
    else {
        panic!("non-finite semantic overlay weight should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::InvalidSemanticOverlay {
            field: "weight",
            ..
        }
    ));
}
