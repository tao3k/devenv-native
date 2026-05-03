use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::studio::router::{GatewayState, StudioState};
use crate::studio::test_support::{commit_all, init_git_repository};
use xiuxian_wendao::analyzers::RegisteredRepository;
use xiuxian_wendao::search::contracts::{UiConfig, UiRepoProjectConfig};

pub(super) struct GatewayFixture {
    pub(super) state: Arc<GatewayState>,
    pub(super) temp_dir: tempfile::TempDir,
}

pub(super) fn make_gateway_fixture() -> Result<GatewayFixture, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let search_plane_root = temp_dir.path().join("search-plane");
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(xiuxian_wendao::analyzers::bootstrap_builtin_registry()?),
        search_plane_root,
    );
    Ok(GatewayFixture {
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio),
        }),
        temp_dir,
    })
}

pub(super) fn configure_repo_project(
    studio: &StudioState,
    repository: &RegisteredRepository,
    plugins: Vec<String>,
) {
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: repository.id.clone(),
            root: repository
                .path
                .as_ref()
                .map(|path| path.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins,
        }],
    });
}

pub(super) fn create_sample_julia_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            "name = \"{package_name}\"\nuuid = \"12345678-1234-1234-1234-123456789abc\"\nversion = \"0.1.0\"\n"
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!(
            r"module {package_name}

export solve, Problem

struct Problem
    x::Int
end

function solve(problem::Problem)
    problem.x
end

end
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

pub(super) fn create_sample_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers"))?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            r"within ;
package {package_name}
end {package_name};
"
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers/PI.mo"),
        format!(
            r"within {package_name}.Controllers;
model PI
  parameter Real k = 1;
  parameter Real Ti = 0.1;
end PI;
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

pub(super) fn create_import_backed_modelica_repo(
    base: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join("modelica_import_backed");
    fs::create_dir_all(repo_dir.join("Modelica/Blocks"))?;
    fs::write(
        repo_dir.join("Modelica/package.mo"),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    fs::write(
        repo_dir.join("Modelica/Blocks/package.mo"),
        "within Modelica;\npackage Blocks\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\nend Blocks;\n",
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

pub(super) fn create_sample_rust_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            package_name.to_ascii_lowercase()
        ),
    )?;
    fs::write(
        repo_dir.join("src/lib.rs"),
        r"pub struct Dataset;

fn scan_rows(dataset: &Dataset) {
    let _ = dataset;
}
",
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn initialize_git_fixture(repo_dir: &Path) {
    init_git_repository(repo_dir);
    commit_all(repo_dir, "seed fixture");
}

pub(super) fn workspace_root() -> PathBuf {
    if let Ok(project_root) = std::env::var("PRJ_ROOT") {
        let candidate = PathBuf::from(project_root);
        if workspace_root_candidate_is_valid(candidate.as_path()) {
            return candidate;
        }
    }

    let candidate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map_or_else(
            || panic!("resolve workspace root from CARGO_MANIFEST_DIR"),
            Path::to_path_buf,
        );
    if workspace_root_candidate_is_valid(candidate.as_path()) {
        return candidate;
    }
    panic!(
        "resolved workspace root candidate `{}` failed marker checks",
        candidate.display()
    )
}

fn workspace_root_candidate_is_valid(candidate: &Path) -> bool {
    candidate.join("Cargo.lock").is_file()
        && candidate
            .join("packages/rust/crates/xiuxian-wendao/Cargo.toml")
            .is_file()
}
