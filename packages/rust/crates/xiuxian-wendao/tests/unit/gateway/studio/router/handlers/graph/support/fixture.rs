use std::sync::Arc;

use tempfile::TempDir;

use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::types::{UiConfig, UiProjectConfig};

pub(crate) struct Fixture {
    pub(crate) state: Arc<GatewayState>,
    pub(crate) _temp_dir: TempDir,
}

pub(crate) fn build_fixture_with_projects(
    docs: &[(&str, &str)],
    projects: Vec<UiProjectConfig>,
) -> Fixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
    for (path, content) in docs {
        let absolute_path = temp_dir.path().join(path);
        if let Some(parent) = absolute_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("create fixture doc parent: {error}"));
        }
        std::fs::write(absolute_path, content)
            .unwrap_or_else(|error| panic!("write fixture doc: {error}"));
    }

    let mut studio_state = StudioState::new();
    studio_state.project_root = temp_dir.path().to_path_buf();
    studio_state.config_root = temp_dir.path().to_path_buf();
    studio_state.seed_eager_configured_owners_for_tests(UiConfig {
        projects,
        repo_projects: Vec::new(),
    });

    Fixture {
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio_state),
        }),
        _temp_dir: temp_dir,
    }
}

pub(crate) fn build_fixture(docs: &[(&str, &str)]) -> Fixture {
    build_fixture_with_projects(
        docs,
        vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec![".".to_string()],
        }],
    )
}
