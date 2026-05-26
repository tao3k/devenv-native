use super::{
    search_strategy_flow_response_bundle_payload_columns,
    search_strategy_flow_response_payload_table_name, search_strategy_flow_table_contract_rows,
};

#[test]
fn rust_search_strategy_flow_contract_manifest_matches_julia_row_shape() {
    let rows = search_strategy_flow_table_contract_rows();

    assert_eq!(rows.len(), 82);
    assert_eq!(rows[0].table_name, "strategy_candidates_request");
    assert_eq!(rows[0].column_name, "candidate_id");
    assert_eq!(rows[0].column_index, 1);
    assert_eq!(rows[0].direction, "request");
    assert!(rows[0].required_column);
    assert!(!rows[0].exact_column_set);

    let response_bundle_rows = rows
        .iter()
        .filter(|row| row.table_name == "search_strategy_flow_response")
        .collect::<Vec<_>>();
    assert_eq!(response_bundle_rows.len(), 4);
    assert_eq!(
        response_bundle_rows
            .iter()
            .map(|row| (
                row.column_name,
                row.column_index,
                row.direction,
                row.required_column,
                row.exact_column_set,
            ))
            .collect::<Vec<_>>(),
        vec![
            ("strategy_candidates_payload", 1, "response", true, true),
            ("strategy_transitions_payload", 2, "response", true, true),
            ("strategy_frontier_payload", 3, "response", true, true),
            (
                "strategy_planner_actions_payload",
                4,
                "response",
                true,
                true,
            ),
        ]
    );
}

#[test]
fn rust_search_strategy_flow_response_payload_routing_is_contract_ordered() {
    assert_eq!(
        search_strategy_flow_response_bundle_payload_columns(),
        [
            "strategy_candidates_payload",
            "strategy_transitions_payload",
            "strategy_frontier_payload",
            "strategy_planner_actions_payload",
        ]
    );
    assert_eq!(
        must(search_strategy_flow_response_payload_table_name(
            "strategy_frontier_payload"
        )),
        "strategy_frontier"
    );
    assert!(
        must_err(search_strategy_flow_response_payload_table_name(
            "unknown_payload"
        ))
        .contains("has no table contract")
    );
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error}"),
    }
}

fn must_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected error, got {value:?}"),
        Err(error) => error,
    }
}
