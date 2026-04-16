use super::*;

#[derive(Default)]
pub(super) struct RepoSearchRequestFilters {
    pub(super) language_filters: HashSet<String>,
    pub(super) path_prefixes: HashSet<String>,
    pub(super) title_filters: HashSet<String>,
    pub(super) tag_filters: HashSet<String>,
    pub(super) filename_filters: HashSet<String>,
}

pub(super) struct TempDirFixture {
    path: PathBuf,
}

impl TempDirFixture {
    pub(super) fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for TempDirFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(super) fn tempdir_or_panic(context: &str) -> TempDirFixture {
    let unique = format!(
        "xiuxian-wendao-flight-repo-search-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("{context}: {error}"))
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&path).unwrap_or_else(|error| panic!("{context}: {error}"));
    TempDirFixture { path }
}

pub(super) fn create_dir_all_or_panic(path: impl AsRef<Path>, context: &str) {
    std::fs::create_dir_all(path).unwrap_or_else(|error| panic!("{context}: {error}"));
}

pub(super) fn write_file_or_panic(path: impl AsRef<Path>, contents: &str, context: &str) {
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{context}: {error}"));
}

pub(super) fn init_git_repo_or_panic(path: impl AsRef<Path>, context: &str) {
    let output = Command::new("git")
        .args(["init", "--quiet"])
        .arg(path.as_ref())
        .output()
        .unwrap_or_else(|error| panic!("{context}: {error}"));
    if output.status.success() {
        return;
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = match (stderr.is_empty(), stdout.is_empty()) {
        (false, true) => stderr,
        (true, false) => stdout,
        (false, false) => format!("{stderr}; stdout: {stdout}"),
        (true, true) => "unknown git error".to_string(),
    };
    panic!("{context}: git init failed: {detail}");
}

pub(super) fn commit_all_or_panic(path: impl AsRef<Path>, message: &str, context: &str) {
    commit_all(path.as_ref(), message);
    let status = Command::new("git")
        .arg("-C")
        .arg(path.as_ref())
        .args(["status", "--short"])
        .output()
        .unwrap_or_else(|error| panic!("{context}: {error}"));
    if status.status.success() {
        return;
    }
    panic!("{context}: git status failed after commit");
}

pub(super) fn repo_document(path: &str, language: &str, contents: &str) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string(),
        language: Some(language.to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes: u64::try_from(contents.len())
            .unwrap_or_else(|error| panic!("document length should fit: {error}")),
        modified_unix_ms: 10,
    }
}

pub(super) fn repo_search_request(
    repo_id: &str,
    query_text: &str,
    limit: usize,
    filters: RepoSearchRequestFilters,
) -> RepoSearchFlightRequest {
    RepoSearchFlightRequest {
        repo_id: repo_id.to_string(),
        query_text: query_text.to_string(),
        limit,
        language_filters: filters.language_filters,
        path_prefixes: filters.path_prefixes,
        title_filters: filters.title_filters,
        tag_filters: filters.tag_filters,
        filename_filters: filters.filename_filters,
    }
}

pub(super) fn string_column<'a>(batch: &'a LanceRecordBatch, column: &str) -> &'a LanceStringArray {
    let Some(column) = batch
        .column_by_name(column)
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
    else {
        panic!("`{column}` should decode as Utf8");
    };
    column
}

pub(super) fn float_column<'a>(batch: &'a LanceRecordBatch, column: &str) -> &'a LanceFloat64Array {
    let Some(column) = batch
        .column_by_name(column)
        .and_then(|column| column.as_any().downcast_ref::<LanceFloat64Array>())
    else {
        panic!("`{column}` should decode as Float64");
    };
    column
}

pub(super) fn first_ticket(flight_info: &FlightInfo, context: &str) -> String {
    let Some(endpoint) = flight_info.endpoint.first() else {
        panic!("{context} should emit one ticket");
    };
    let Some(ticket) = endpoint.ticket.as_ref() else {
        panic!("{context} should emit one ticket");
    };
    String::from_utf8_lossy(ticket.ticket.as_ref()).into_owned()
}

pub(super) async fn repo_search_batch_or_panic(
    provider: &StudioRepoSearchFlightRouteProvider,
    request: &RepoSearchFlightRequest,
    context: &str,
) -> LanceRecordBatch {
    provider
        .repo_search_batch(request)
        .await
        .unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn populate_search_headers(
    metadata: &mut tonic::metadata::MetadataMap,
    query: &str,
    limit: usize,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2".parse()
            .unwrap_or_else(|error| panic!("schema metadata: {error}")),
    );
    metadata.insert(
        WENDAO_SEARCH_QUERY_HEADER,
        query
            .parse()
            .unwrap_or_else(|error| panic!("query metadata: {error}")),
    );
    metadata.insert(
        WENDAO_SEARCH_LIMIT_HEADER,
        limit
            .to_string()
            .parse()
            .unwrap_or_else(|error| panic!("limit metadata: {error}")),
    );
}

pub(super) fn populate_markdown_analysis_headers(
    metadata: &mut tonic::metadata::MetadataMap,
    path: &str,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        "v2".parse()
            .unwrap_or_else(|error| panic!("schema metadata: {error}")),
    );
    metadata.insert(
        WENDAO_ANALYSIS_PATH_HEADER,
        path.parse()
            .unwrap_or_else(|error| panic!("analysis path metadata: {error}")),
    );
}

pub(super) fn populate_code_ast_analysis_headers(
    metadata: &mut tonic::metadata::MetadataMap,
    path: &str,
    repo_id: &str,
    line_hint: Option<usize>,
) {
    populate_markdown_analysis_headers(metadata, path);
    metadata.insert(
        WENDAO_ANALYSIS_REPO_HEADER,
        repo_id
            .parse()
            .unwrap_or_else(|error| panic!("analysis repo metadata: {error}")),
    );
    if let Some(line_hint) = line_hint {
        metadata.insert(
            WENDAO_ANALYSIS_LINE_HEADER,
            line_hint
                .to_string()
                .parse()
                .unwrap_or_else(|error| panic!("analysis line metadata: {error}")),
        );
    }
}

pub(super) fn test_studio_state(search_plane_root: PathBuf) -> StudioState {
    let plugin_registry = Arc::new(
        bootstrap_builtin_registry().unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = SearchPlaneService::new(search_plane_root.clone());
    StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        search_plane_root.clone(),
        search_plane_root,
        search_plane,
    )
}
