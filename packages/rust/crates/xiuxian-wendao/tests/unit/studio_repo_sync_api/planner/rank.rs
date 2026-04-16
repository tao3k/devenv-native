use super::support::*;

#[tokio::test]
async fn docs_planner_rank_endpoint_returns_priority_sorted_gaps() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve, explain\nsolve() = nothing\nexplain() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("orphan_demo.jl"),
        "println(\"detached example\")\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("orphan.md"), "# orphan\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) =
        request_json(router, "/api/docs/planner-rank?repo=gateway-sync&limit=4").await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_rank_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_rank_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let (_temp_dir, router) = modelica_nodocs_router("modelica-gateway-rank")?;

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-rank?repo=modelica-gateway-rank&gap_kind=symbol_reference_without_documentation&page_kind=reference&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("planner-rank payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "planner-rank endpoint should return at least one ranked gap hit"
    );
    assert!(
        hits.len() <= 4,
        "planner-rank endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().all(|hit| {
            hit.get("reasons")
                .and_then(Value::as_array)
                .is_some_and(|reasons| !reasons.is_empty())
        }),
        "planner-rank endpoint should keep deterministic score explanations"
    );
    assert!(
        hits.iter().all(|hit| hit_gap_matches_needle(hit, "NoDocs")),
        "planner-rank endpoint hits should stay anchored to the injected no-doc target"
    );
    assert!(
        hits.windows(2)
            .all(|window| planner_rank_key(&window[0]) <= planner_rank_key(&window[1])),
        "planner-rank endpoint hits should stay in deterministic priority order"
    );
    assert_studio_json_snapshot("docs_planner_rank_endpoint_modelica_json", payload);
    Ok(())
}
