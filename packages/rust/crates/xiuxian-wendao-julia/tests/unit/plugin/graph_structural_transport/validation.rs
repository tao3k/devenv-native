#[test]
fn validate_graph_structural_request_batches_accepts_staged_shapes() {
    let rerank = structural_rerank_request_batch();
    let filter = constraint_filter_request_batch();

    assert!(
        validate_graph_structural_request_batches(
            GraphStructuralRouteKind::StructuralRerank,
            &[rerank]
        )
        .is_ok()
    );
    assert!(
        validate_graph_structural_request_batches(
            GraphStructuralRouteKind::ConstraintFilter,
            &[filter]
        )
        .is_ok()
    );
}

#[test]
fn validate_graph_structural_response_batches_accepts_staged_shapes() {
    let rerank = structural_rerank_response_batch();
    let filter = constraint_filter_response_batch();

    assert!(
        validate_graph_structural_response_batches(
            GraphStructuralRouteKind::StructuralRerank,
            &[rerank]
        )
        .is_ok()
    );
    assert!(
        validate_graph_structural_response_batches(
            GraphStructuralRouteKind::ConstraintFilter,
            &[filter]
        )
        .is_ok()
    );
}

#[test]
fn validate_graph_structural_response_batches_rejects_wrong_shape() {
    let error = validate_graph_structural_response_batches(
        GraphStructuralRouteKind::ConstraintFilter,
        &[structural_rerank_response_batch()],
    )
    .err_or_panic("wrong graph-structural response shape must fail");
    assert!(
        error
            .to_string()
            .contains("Julia graph-structural response contract"),
        "unexpected error: {error}"
    );
}
