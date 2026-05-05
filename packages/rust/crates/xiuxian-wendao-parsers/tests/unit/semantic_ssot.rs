use std::fs;
use std::path::Path;
use tempfile::tempdir;
use xiuxian_wendao_parsers::{
    SemanticScopeRequest, load_semantic_repository, parse_semantic_object,
    semantic_projection_source_revision, semantic_scope_bundle,
};

#[test]
fn semantic_parser_accepts_object_frontmatter() {
    let object = parse_semantic_object(
        "objects/component/test.md",
        &semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "",
        ),
    )
    .unwrap_or_else(|error| panic!("semantic object should parse: {error}"));

    assert_eq!(object.id, "component.test");
    assert_eq!(object.relations.len(), 0);
    assert_eq!(object.source_path, Path::new("objects/component/test.md"));
}

#[test]
fn semantic_repository_validates_relation_targets_and_projection_sources() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "  - kind: depends_on\n    target: decision.missing\n",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "  - component.missing\n",
            "source_revision: stale-fixture\n",
            "projection_revision: test.v1\n",
            "staleness: stale\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
    );

    let repository = load_semantic_repository(temp.path());
    let messages = repository
        .report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("decision.missing")),
        "missing relation target should be reported: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("component.missing")),
        "missing projection source should be reported: {messages:?}"
    );
}

#[test]
fn semantic_scope_bundle_includes_active_relation_neighborhood() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/task/pilot.md"),
        semantic_object_fixture(
            "task.pilot",
            "task",
            "Pilot Task",
            "active",
            "  - kind: constrains\n    target: invariant.runtime\n",
        ),
    );
    write_file(
        temp.path().join("objects/invariant/runtime.md"),
        semantic_object_fixture(
            "invariant.runtime",
            "invariant",
            "Runtime Invariant",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(
            &["task.pilot", "invariant.runtime"],
            "stale-fixture",
            "stale",
        ),
    );
    refresh_projection_as_fresh(temp.path(), &["task.pilot", "invariant.runtime"]);

    let repository = load_semantic_repository(temp.path());
    assert!(
        repository.report.is_success(),
        "repository should validate: {:?}",
        repository.report.issues
    );

    let bundle = semantic_scope_bundle(
        &repository,
        &SemanticScopeRequest {
            task_id: Some("task.pilot".to_string()),
            object_ids: Vec::new(),
        },
    );

    assert_eq!(
        bundle
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect::<Vec<_>>(),
        vec!["invariant.runtime", "task.pilot"]
    );
    assert_eq!(bundle.affected_invariants, vec!["invariant.runtime"]);
    assert_eq!(bundle.projection_revision, "test.v1");
    assert!(bundle.projection_source_revision.is_some());
    assert_eq!(
        serde_json::to_value(&bundle.projection_staleness).expect("serialize staleness"),
        serde_json::json!("fresh")
    );
}

#[test]
fn semantic_repository_flags_unmarked_stale_projection_source_revision() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(&["component.test"], "outdated", "fresh"),
    );

    let repository = load_semantic_repository(temp.path());
    let messages = repository
        .report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("source revision is stale")),
        "stale fresh projection should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_accepts_explicitly_stale_projection_source_revision() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(&["component.test"], "outdated", "stale"),
    );

    let repository = load_semantic_repository(temp.path());

    assert!(
        repository.report.is_success(),
        "explicit stale projection should validate: {:?}",
        repository.report.issues
    );
}

fn semantic_object_fixture<'a>(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    relations: &'a str,
) -> String {
    format!(
        concat!(
            "---\n",
            "id: {id}\n",
            "kind: {kind}\n",
            "title: {title}\n",
            "status: {status}\n",
            "confidence:\n",
            "  score: 1.0\n",
            "  source: human_signed\n",
            "owners:\n",
            "  - scope: packages/rust/crates/xiuxian-wendao-parsers\n",
            "    role: parser_owner\n",
            "provenance:\n",
            "  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md\n",
            "  recorded_by: codex\n",
            "  recorded_at: \"2026-05-05\"\n",
            "verification:\n",
            "  required:\n",
            "    - direnv exec . wendao-client lint semantic semantic\n",
            "relations:\n",
            "{relations}",
            "---\n",
            "# {title}\n",
            "\n",
            "Fixture body.\n",
        ),
        id = id,
        kind = kind,
        title = title,
        status = status,
        relations = if relations.is_empty() {
            "  []\n"
        } else {
            relations
        },
    )
}

fn write_file(path: impl AsRef<Path>, content: impl AsRef<str>) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("fixture directory should be created: {error}"));
    }
    fs::write(path, content.as_ref())
        .unwrap_or_else(|error| panic!("fixture file should be written: {error}"));
}

fn semantic_projection_fixture(
    source_objects: &[&str],
    source_revision: &str,
    staleness: &str,
) -> String {
    let source_objects = source_objects
        .iter()
        .map(|object_id| format!("  - {object_id}\n"))
        .collect::<String>();
    format!(
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "{source_objects}",
            "source_revision: {source_revision}\n",
            "projection_revision: test.v1\n",
            "staleness: {staleness}\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
        source_objects = source_objects,
        source_revision = source_revision,
        staleness = staleness,
    )
}

fn refresh_projection_as_fresh(root: &Path, source_objects: &[&str]) {
    let stale_repository = load_semantic_repository(root);
    let projection = stale_repository
        .projections
        .first()
        .expect("projection fixture should load");
    let source_revision = semantic_projection_source_revision(&stale_repository, projection)
        .expect("projection source revision should compute");
    write_file(
        root.join("projections/llm-compression.md"),
        semantic_projection_fixture(source_objects, &source_revision, "fresh"),
    );
}
