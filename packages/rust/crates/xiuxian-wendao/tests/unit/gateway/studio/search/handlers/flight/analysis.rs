use xiuxian_wendao_runtime::transport::{
    ANALYSIS_CODE_AST_ROUTE, ANALYSIS_REFINE_DOC_ROUTE, ANALYSIS_REPO_DOC_COVERAGE_ROUTE,
    ANALYSIS_REPO_OVERVIEW_ROUTE, ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_MARKDOWN_ROUTE, ANALYSIS_REPO_INDEX_ROUTE, ANALYSIS_REPO_INDEX_STATUS_ROUTE,
    ANALYSIS_REPO_SYNC_ROUTE,
};

use crate::gateway::studio::search::handlers::tests::{
    publish_repo_entity_index, sample_repo_analysis,
};

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs, make_gateway_state_with_repo,
    populate_markdown_analysis_headers, populate_repo_index_headers,
    populate_repo_index_status_headers, populate_repo_sync_headers,
};
use super::{
    populate_code_ast_analysis_headers, populate_refine_doc_headers,
    populate_repo_doc_coverage_headers, populate_repo_overview_headers,
    populate_repo_projected_page_index_tree_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_markdown_analysis_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "docs/analysis.md",
        "# Analysis Kernel\n\n## Inputs\n- [ ] Parse markdown\n",
    )]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_MARKDOWN_ROUTE,
        "markdown analysis route",
        |metadata| populate_markdown_analysis_headers(metadata, "kernel/docs/analysis.md"),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_code_ast_analysis_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_CODE_AST_ROUTE,
        "code-AST analysis route",
        |metadata| populate_code_ast_analysis_headers(metadata, "src/lib.jl", "demo", Some(3)),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_overview_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        ("README.md", "# Demo\n\nPackage docs.\n"),
        (
            "docs/solve.md",
            "# solve\n\nDocument the exported function.\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\n\"solve docs\"\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    publish_repo_entity_index(&fixture.state.studio, "demo", &sample_repo_analysis("demo")).await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_OVERVIEW_ROUTE,
        "repo overview route",
        |metadata| populate_repo_overview_headers(metadata, "demo"),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_index_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_INDEX_ROUTE,
        "repo index route",
        |metadata| populate_repo_index_headers(metadata, Some("demo"), true),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_index_status_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_INDEX_STATUS_ROUTE,
        "repo index status route",
        |metadata| populate_repo_index_status_headers(metadata, Some("demo")),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_sync_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_SYNC_ROUTE,
        "repo sync route",
        |metadata| populate_repo_sync_headers(metadata, "demo", Some("status")),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_doc_coverage_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        ("README.md", "# Demo\n\nPackage docs.\n"),
        (
            "docs/solve.md",
            "# solve\n\nDocument the exported function.\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\n\"solve docs\"\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_DOC_COVERAGE_ROUTE,
        "repo doc coverage route",
        |metadata| populate_repo_doc_coverage_headers(metadata, "demo", None),
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_repo_projected_page_index_tree_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        ("README.md", "# Demo\n\nPackage docs.\n"),
        (
            "docs/solve.md",
            "# solve\n\nDocument the exported function.\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\n\"solve docs\"\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "repo projected page-index tree route",
        |metadata| {
            populate_repo_projected_page_index_tree_headers(
                metadata,
                "demo",
                "repo:demo:projection:reference:doc:repo:demo:doc:docs/solve.md",
            );
        },
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_refine_doc_routes() {
    let fixture = make_gateway_state_with_repo(&[
        (
            "Project.toml",
            "name = \"Demo\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
        ),
        (
            "src/lib.jl",
            "module Demo\nexport solve\n\"solve docs\"\nsolve(x) = x + 1\nend\n",
        ),
    ]);
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        ANALYSIS_REFINE_DOC_ROUTE,
        "refine-doc route",
        |metadata| {
            populate_refine_doc_headers(metadata, "demo", "repo:demo:symbol:Demo.solve", None);
        },
    )
    .await;
}
