use super::{
    SemanticChangeIntentFixture, load_semantic_repository, refresh_projection_as_fresh,
    semantic_change_intent_fixture_with_lifecycle_targets,
    semantic_change_intent_fixture_with_status_transitions, semantic_object_fixture,
    semantic_projection_fixture, tempdir, write_file,
};

#[path = "transitions/accepted.rs"]
mod accepted;
#[path = "transitions/status_rules.rs"]
mod status_rules;
#[path = "transitions/target_rules.rs"]
mod target_rules;
