use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use xiuxian_wendao_parsers::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, SemanticScopeRequest, load_semantic_repository,
    parse_semantic_object, parse_semantic_scope_metadata_envelope_json,
    semantic_projection_freshness_policy_report, semantic_projection_refresh_plan_report,
    semantic_projection_source_revision, semantic_scope_bundle, semantic_scope_metadata_envelope,
    semantic_scope_metadata_envelope_to_vec,
};

mod candidates;
mod invalid_references;
mod parse_repository;
mod projection;
mod scope;
mod transitions;

fn semantic_object_fixture(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    relations: &str,
) -> String {
    semantic_object_fixture_with_confidence(id, kind, title, status, "human_signed", relations)
}

fn semantic_object_fixture_with_confidence(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    confidence_source: &str,
    relations: &str,
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
            "  source: {confidence_source}\n",
            "owners:\n",
            "  - scope: packages/rust/crates/xiuxian-wendao-parsers\n",
            "    role: parser_owner\n",
            "provenance:\n",
            "  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md\n",
            "  recorded_by: codex\n",
            "  recorded_at: \"2026-05-05\"\n",
            "verification:\n",
            "  required:\n",
            "    - direnv exec . wendao-client lint semantic\n",
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
        confidence_source = confidence_source,
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
    let mut rendered_source_objects = String::new();
    for object_id in source_objects {
        writeln!(&mut rendered_source_objects, "  - {object_id}")
            .unwrap_or_else(|error| panic!("render projection source object: {error}"));
    }
    format!(
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "{rendered_source_objects}",
            "source_revision: {source_revision}\n",
            "projection_revision: test.v1\n",
            "staleness: {staleness}\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
        rendered_source_objects = rendered_source_objects,
        source_revision = source_revision,
        staleness = staleness,
    )
}

fn refresh_projection_as_fresh(root: &Path, source_objects: &[&str]) {
    let stale_repository = load_semantic_repository(root);
    let Some(projection) = stale_repository.projections.first() else {
        panic!("projection fixture should load");
    };
    let source_revision = semantic_projection_source_revision(&stale_repository, projection)
        .unwrap_or_else(|| panic!("projection source revision should compute"));
    write_file(
        root.join("projections/llm-compression.md"),
        semantic_projection_fixture(source_objects, &source_revision, "fresh"),
    );
}

fn semantic_change_intent_fixture(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
) -> String {
    semantic_change_intent_fixture_with_candidates(
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        &[],
    )
}

fn semantic_change_intent_fixture_with_candidates(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
    candidate_suggestions: &[&str],
) -> String {
    semantic_change_intent_fixture_with_status_transitions(
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        candidate_suggestions,
        &[],
    )
}

fn semantic_change_intent_fixture_with_status_transitions(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
    candidate_suggestions: &[&str],
    status_transitions: &[(&str, &str, &str)],
) -> String {
    semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        candidate_suggestions,
        status_transitions,
        promotion_targets: &[],
        demotion_targets: &[],
    })
}

#[derive(Clone, Copy)]
struct SemanticChangeIntentFixture<'a> {
    touched_object: &'a str,
    affected_invariant: &'a str,
    relation_source: &'a str,
    relation_target: &'a str,
    projection: &'a str,
    candidate_suggestions: &'a [&'a str],
    status_transitions: &'a [(&'a str, &'a str, &'a str)],
    promotion_targets: &'a [&'a str],
    demotion_targets: &'a [&'a str],
}

fn semantic_change_intent_fixture_with_lifecycle_targets(
    fixture: SemanticChangeIntentFixture<'_>,
) -> String {
    let mut rendered_candidate_suggestions = String::new();
    if fixture.candidate_suggestions.is_empty() {
        rendered_candidate_suggestions.push_str("[]\n");
    } else {
        rendered_candidate_suggestions.push('\n');
        for object_id in fixture.candidate_suggestions {
            writeln!(&mut rendered_candidate_suggestions, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render candidate suggestion: {error}"));
        }
    }
    let mut rendered_status_transitions = String::new();
    if fixture.status_transitions.is_empty() {
        rendered_status_transitions.push_str("[]\n");
    } else {
        rendered_status_transitions.push('\n');
        for (object_id, from, to) in fixture.status_transitions {
            writeln!(
                &mut rendered_status_transitions,
                "  - object_id: {object_id}\n    from: {from}\n    to: {to}"
            )
            .unwrap_or_else(|error| panic!("render status transition: {error}"));
        }
    }
    let mut rendered_promotion_targets = String::new();
    if fixture.promotion_targets.is_empty() {
        rendered_promotion_targets.push_str("[]\n");
    } else {
        rendered_promotion_targets.push('\n');
        for object_id in fixture.promotion_targets {
            writeln!(&mut rendered_promotion_targets, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render promotion target: {error}"));
        }
    }
    let mut rendered_demotion_targets = String::new();
    if fixture.demotion_targets.is_empty() {
        rendered_demotion_targets.push_str("[]\n");
    } else {
        rendered_demotion_targets.push('\n');
        for object_id in fixture.demotion_targets {
            writeln!(&mut rendered_demotion_targets, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render demotion target: {error}"));
        }
    }
    format!(
        concat!(
            "---\n",
            "type: semantic_change_intent\n",
            "id: change.semantic-ssot.test\n",
            "title: Semantic SSOT Test Change\n",
            "status: active\n",
            "touched_objects:\n",
            "  - {touched_object}\n",
            "changed_relations:\n",
            "  - source: {relation_source}\n",
            "    kind: validates\n",
            "    target: {relation_target}\n",
            "    action: add\n",
            "status_transitions: {rendered_status_transitions}",
            "promotion_targets: {rendered_promotion_targets}",
            "demotion_targets: {rendered_demotion_targets}",
            "affected_invariants:\n",
            "  - {affected_invariant}\n",
            "required_validations:\n",
            "  - direnv exec . cargo test -p xiuxian-wendao-parsers semantic -- --nocapture\n",
            "projections_to_refresh:\n",
            "  - {projection}\n",
            "candidate_suggestions: {rendered_candidate_suggestions}",
            "---\n",
            "# Semantic SSOT Test Change\n",
        ),
        touched_object = fixture.touched_object,
        affected_invariant = fixture.affected_invariant,
        relation_source = fixture.relation_source,
        relation_target = fixture.relation_target,
        projection = fixture.projection,
        rendered_candidate_suggestions = rendered_candidate_suggestions,
        rendered_status_transitions = rendered_status_transitions,
        rendered_promotion_targets = rendered_promotion_targets,
        rendered_demotion_targets = rendered_demotion_targets,
    )
}
