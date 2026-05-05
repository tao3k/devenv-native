use std::sync::Arc;

use tempfile::TempDir;

use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::router::{
    GatewayState, GraphIndexCacheEntry, GraphSourceSignature, StudioState,
};
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(crate) struct Fixture {
    pub(crate) state: Arc<GatewayState>,
    pub(crate) _temp_dir: TempDir,
}

pub(crate) fn build_fixture_with_projects(
    docs: &[(&str, &str)],
    projects: &[UiProjectConfig],
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
    studio_state.seed_configured_owners_for_tests(
        UiConfig {
            projects: projects.to_vec(),
            repo_projects: Vec::new(),
        },
        false,
    );

    let include_dirs = graph_include_dirs(temp_dir.path(), projects);
    let graph_index =
        LinkGraphIndex::build_with_filters(temp_dir.path(), include_dirs.as_slice(), &[])
            .unwrap_or_else(|error| panic!("build fixture graph index: {error}"));
    let mut graph_guard = studio_state
        .graph_index
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *graph_guard = Some(GraphIndexCacheEntry {
        index: Arc::new(graph_index),
        source_signature: GraphSourceSignature::default(),
    });
    drop(graph_guard);

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

fn graph_include_dirs(project_root: &std::path::Path, projects: &[UiProjectConfig]) -> Vec<String> {
    let mut include_dirs = Vec::new();
    for project in projects {
        let project_base = project_root.join(project.root.trim());
        for dir in &project.dirs {
            let candidate = project_base.join(dir.trim());
            let Ok(relative) = candidate.strip_prefix(project_root) else {
                continue;
            };
            let normalized = relative
                .to_string_lossy()
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_string();
            include_dirs.push(if normalized.is_empty() {
                ".".to_string()
            } else {
                normalized
            });
        }
    }
    include_dirs.sort();
    include_dirs.dedup();
    include_dirs
}

pub(crate) fn build_fixture(docs: &[(&str, &str)]) -> Fixture {
    build_fixture_with_projects(
        docs,
        &[UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec![".".to_string()],
        }],
    )
}
