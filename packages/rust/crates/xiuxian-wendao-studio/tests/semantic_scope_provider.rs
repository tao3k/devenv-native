#![cfg(feature = "zhenfa-router")]

//! Studio Flight semantic-scope provider integration smoke tests.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightDescriptor, FlightInfo};
use serde_json::Value;
use tonic::Request;
use xiuxian_wendao::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
use xiuxian_wendao_server::transport::{
    ANALYSIS_SEMANTIC_SCOPE_ROUTE, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER, flight_descriptor_path,
};
use xiuxian_wendao_studio::studio::build_studio_flight_service_for_roots;

const TEST_SCHEMA_VERSION: &str = "semantic-scope-studio-smoke";

#[tokio::test]
async fn semantic_scope_provider_serves_repo_native_bundle_through_studio_flight_service() {
    let temp_dir = TempDirFixture::new("semantic-scope-studio-smoke temp dir");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    seed_project_semantic_fixture(&project_root);

    let search_plane = Arc::new(SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:semantic-scope-studio-smoke"),
        SearchMaintenancePolicy::default(),
    ));
    let service = build_studio_flight_service_for_roots(
        search_plane,
        project_root.clone(),
        project_root,
        TEST_SCHEMA_VERSION,
        3,
    )
    .unwrap_or_else(|error| panic!("studio semantic-scope Flight service should build: {error}"));

    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(ANALYSIS_SEMANTIC_SCOPE_ROUTE)
            .unwrap_or_else(|error| panic!("semantic-scope descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    request.metadata_mut().insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        TEST_SCHEMA_VERSION
            .parse()
            .unwrap_or_else(|error| panic!("schema metadata value: {error}")),
    );
    request.metadata_mut().insert(
        WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
        "task.semantic-scope-pilot"
            .parse()
            .unwrap_or_else(|error| panic!("task metadata value: {error}")),
    );

    let flight_info = service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("semantic-scope get_flight_info: {error}"))
        .into_inner();
    assert_eq!(first_ticket(&flight_info), ANALYSIS_SEMANTIC_SCOPE_ROUTE);
    assert_eq!(flight_info.total_records, 2);

    let metadata: Value = serde_json::from_slice(&flight_info.app_metadata)
        .unwrap_or_else(|error| panic!("semantic-scope app metadata should decode: {error}"));
    let bundle = &metadata["semanticScopeBundle"];
    assert_eq!(bundle["projection_revision"], "semantic-scope-test");
    assert_eq!(bundle["projection_staleness"], "stale");
    assert_eq!(
        bundle_object_status(bundle, "task.semantic-scope-pilot"),
        Some("candidate")
    );
    assert_eq!(
        bundle_object_status(bundle, "component.demo"),
        Some("active")
    );
    assert_eq!(
        bundle["change_intents"].as_array().map(std::vec::Vec::len),
        Some(1)
    );

    let sql_guard = &metadata["semanticSqlGuardEvidence"];
    assert_eq!(sql_guard["guardId"], "semantic_sql.projection_freshness");
    assert_eq!(sql_guard["status"], "review_required");
    assert_eq!(sql_guard["failingRowCount"], 1);

    let projection_policy = &metadata["semanticProjectionPolicyEvidence"];
    assert_eq!(
        projection_policy["policyId"],
        "semantic_projection.required_refresh_targets"
    );
    assert_eq!(projection_policy["status"], "review_required");
    assert_eq!(projection_policy["failingProjectionCount"], 1);
    assert_eq!(
        projection_policy["projections"]
            .as_array()
            .map(std::vec::Vec::len),
        Some(1)
    );
}

fn first_ticket(flight_info: &FlightInfo) -> String {
    let Some(endpoint) = flight_info.endpoint.first() else {
        panic!("semantic-scope route should emit one endpoint");
    };
    let Some(ticket) = endpoint.ticket.as_ref() else {
        panic!("semantic-scope route should emit one ticket");
    };
    String::from_utf8_lossy(ticket.ticket.as_ref()).into_owned()
}

fn bundle_object_status<'a>(bundle: &'a Value, object_id: &str) -> Option<&'a str> {
    bundle["objects"]
        .as_array()?
        .iter()
        .find(|object| object["id"] == object_id)?
        .get("status")?
        .as_str()
}

fn seed_project_semantic_fixture(project_root: &Path) {
    create_semantic_fixture_dirs(project_root);
    write_semantic_fixture_wendao_config(project_root);
    write_semantic_fixture_component(project_root);
    write_semantic_fixture_invariant(project_root);
    write_semantic_fixture_candidate_task(project_root);
    write_semantic_fixture_projection(project_root);
    write_semantic_fixture_change_intent(project_root);
}

fn create_semantic_fixture_dirs(project_root: &Path) {
    create_dir_all_or_panic(
        project_root.join("semantic/objects/component"),
        "create component object directory",
    );
    create_dir_all_or_panic(
        project_root.join("semantic/objects/invariant"),
        "create invariant object directory",
    );
    create_dir_all_or_panic(
        project_root.join("semantic/objects/task"),
        "create task object directory",
    );
    create_dir_all_or_panic(
        project_root.join("semantic/change-intents"),
        "create change-intent directory",
    );
    create_dir_all_or_panic(
        project_root.join("semantic/projections"),
        "create projection directory",
    );
}

fn write_semantic_fixture_wendao_config(project_root: &Path) {
    write_file_or_panic(
        project_root.join("wendao.toml"),
        r#"
[link_graph.projects.kernel]
root = "."
dirs = ["semantic"]
"#,
        "write wendao.toml",
    );
}

fn write_semantic_fixture_component(project_root: &Path) {
    write_file_or_panic(
        project_root.join("semantic/objects/component/demo.md"),
        r"---
id: component.demo
kind: component
title: Demo Component
status: active
confidence:
  score: 0.92
  source: verified
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: runtime-provider
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Component
",
        "write component object",
    );
}

fn write_semantic_fixture_invariant(project_root: &Path) {
    write_file_or_panic(
        project_root.join("semantic/objects/invariant/authority.md"),
        r"---
id: invariant.demo-authority
kind: invariant
title: Demo Authority Invariant
status: active
confidence:
  score: 0.95
  source: verified
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: runtime-provider
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Authority Invariant
",
        "write invariant object",
    );
}

fn write_semantic_fixture_candidate_task(project_root: &Path) {
    write_file_or_panic(
        project_root.join("semantic/objects/task/pilot.md"),
        r"---
id: task.semantic-scope-pilot
kind: task
title: Semantic Scope Pilot
status: candidate
confidence:
  score: 0.8
  source: llm_suggested
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: route-adapter
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations:
  - kind: implements
    target: component.demo
---

# Semantic Scope Pilot
",
        "write task object",
    );
}

fn write_semantic_fixture_projection(project_root: &Path) {
    write_file_or_panic(
        project_root.join("semantic/projections/llm-compression.md"),
        r"---
type: semantic_projection
projection: llm_compression
source_objects:
  - component.demo
  - task.semantic-scope-pilot
source_revision: stale-fixture
projection_revision: semantic-scope-test
staleness: stale
status: active
---

# LLM Compression
",
        "write projection",
    );
}

fn write_semantic_fixture_change_intent(project_root: &Path) {
    write_file_or_panic(
        project_root.join("semantic/change-intents/semantic-scope-pilot.md"),
        r"---
type: semantic_change_intent
id: change.semantic-scope-pilot
title: Semantic Scope Pilot
status: active
touched_objects:
  - task.semantic-scope-pilot
changed_relations:
  - source: task.semantic-scope-pilot
    kind: implements
    target: component.demo
    action: add
affected_invariants:
  - invariant.demo-authority
required_validations:
  - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
projections_to_refresh:
  - llm_compression
candidate_suggestions:
  - task.semantic-scope-pilot
---

# Semantic Scope Pilot Change
",
        "write change intent",
    );
}

fn create_dir_all_or_panic(path: impl AsRef<Path>, context: &str) {
    std::fs::create_dir_all(path).unwrap_or_else(|error| panic!("{context}: {error}"));
}

fn write_file_or_panic(path: impl AsRef<Path>, contents: &str, context: &str) {
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{context}: {error}"));
}

struct TempDirFixture {
    path: PathBuf,
}

impl TempDirFixture {
    fn new(context: &str) -> Self {
        let unique = format!(
            "xiuxian-wendao-studio-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|error| panic!("{context}: {error}"))
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).unwrap_or_else(|error| panic!("{context}: {error}"));
        Self { path }
    }

    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for TempDirFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
