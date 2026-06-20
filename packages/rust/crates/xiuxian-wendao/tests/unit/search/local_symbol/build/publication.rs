use crate::search::local_symbol::build::ensure_local_symbol_index_started;
use crate::search::local_symbol::search_local_symbols;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
};

use super::support::{
    assert_no_local_symbol_lance_tables, demo_projects, incremental_service,
    start_local_symbol_index, wait_for_local_symbol_ready, write_demo_source,
};

#[tokio::test]
async fn local_symbol_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    write_demo_source(project_root.as_path(), "src/lib.rs", "fn alpha() {}\n");
    write_demo_source(project_root.as_path(), "src/extra.rs", "fn gamma() {}\n");
    let projects = demo_projects();
    let service = incremental_service(
        project_root.as_path(),
        storage_root.as_path(),
        "xiuxian:test:search_plane:local-symbol-incremental",
    );

    start_local_symbol_index(&service, project_root.as_path(), &projects).await;

    let initial_gamma = search_local_symbols(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma: {error}"));
    assert!(initial_gamma.is_empty());
    let initial_alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert!(initial_alpha.is_empty());

    write_demo_source(project_root.as_path(), "src/lib.rs", "fn beta() {}\n");
    ensure_local_symbol_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_local_symbol_ready(&service, Some(1)).await;

    let gamma = search_local_symbols(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma after refresh: {error}"));
    assert!(gamma.is_empty());
    let beta = search_local_symbols(&service, "beta", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta after refresh: {error}"));
    assert!(beta.is_empty());
    let alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha after refresh: {error}"));
    assert!(alpha.is_empty());
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_else(|| panic!("local symbol active epoch"));
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert!(
        !table_names.is_empty(),
        "expected local symbol partition tables"
    );
    for table_name in table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing local symbol parquet export for {table_name}"
        );
    }
    assert_no_local_symbol_lance_tables(&service);
}

#[tokio::test]
async fn local_symbol_build_writes_partitioned_epoch_tables_for_multiple_scopes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    write_demo_source(
        project_root.as_path(),
        "packages/alpha/src/lib.rs",
        "fn alpha() {}\n",
    );
    write_demo_source(
        project_root.as_path(),
        "packages/beta/src/lib.rs",
        "fn beta() {}\n",
    );
    let projects = vec![crate::search::contracts::SearchProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec!["packages/alpha".to_string(), "packages/beta".to_string()],
    }];
    let service = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-partitioned-build"),
        SearchMaintenancePolicy::default(),
    );

    start_local_symbol_index(&service, project_root.as_path(), &projects).await;

    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_default();
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert_eq!(table_names.len(), 2);
    for table_name in &table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing local symbol parquet export for {table_name}"
        );
    }
    assert_no_local_symbol_lance_tables(&service);

    let alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert!(alpha.is_empty());

    let beta = search_local_symbols(&service, "beta", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta: {error}"));
    assert!(beta.is_empty());
}

#[tokio::test]
async fn local_symbol_build_with_no_supported_sources_publishes_empty_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(&project_root)
        .unwrap_or_else(|error| panic!("create workspace root: {error}"));
    let projects = demo_projects();
    let service = incremental_service(
        project_root.as_path(),
        storage_root.as_path(),
        "xiuxian:test:search_plane:local-symbol-empty-epoch",
    );

    start_local_symbol_index(&service, project_root.as_path(), &projects).await;

    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_else(|| panic!("local symbol active epoch"));
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert!(
        !table_names.is_empty(),
        "expected empty local symbol publication"
    );
    for table_name in &table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing empty local symbol parquet export for {table_name}"
        );
    }

    let results = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query empty local symbol epoch: {error}"));
    assert!(results.is_empty());
}
