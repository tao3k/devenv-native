use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::studio::test_support::{commit_all, init_git_repository};
use xiuxian_wendao::analyzers::RepositoryAnalysisOutput;
use xiuxian_wendao::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};

use super::build_code_search_response;

pub(super) async fn publish_repository_snapshot(
    studio: &crate::studio::StudioState,
    repo_id: &str,
    analysis: RepositoryAnalysisOutput,
    documents: Vec<RepoCodeDocument>,
) {
    let analysis = Arc::new(analysis);
    studio
        .search_plane
        .publish_repo_entities_with_revision(repo_id, analysis.as_ref(), &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo entities for `{repo_id}`: {error}"));
    studio
        .search_plane
        .publish_repo_content_chunks_with_revision(repo_id, &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks for `{repo_id}`: {error}"));
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: repo_id.to_string(),
            analysis: Arc::clone(&analysis),
        }));
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: repo_id.to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("fixture".to_string()),
        updated_at: Some("2026-04-09T00:00:00Z".to_string()),
        attempt_count: 1,
    });
}

pub(super) fn repo_code_document(
    repo_root: &Path,
    file_path: impl AsRef<Path>,
    language: &str,
) -> Result<RepoCodeDocument, Box<dyn std::error::Error>> {
    let file_path = file_path.as_ref();
    let contents = fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(RepoCodeDocument {
        path: relative_path,
        language: Some(language.to_string()),
        size_bytes: u64::try_from(contents.len()).unwrap_or(u64::MAX),
        modified_unix_ms: 0,
        contents: Arc::<str>::from(contents),
    })
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
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

pub(super) fn create_sample_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers"))?;
    fs::write(repo_dir.join("package.order"), "Controllers\n")?;
    fs::write(
        repo_dir.join("package.mo"),
        format!("within;\npackage {package_name}\nend {package_name};\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("package.mo"),
        format!("within {package_name};\npackage Controllers\nend Controllers;\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("PI.mo"),
        format!("within {package_name}.Controllers;\nmodel PI\n  parameter Real k = 1;\nend PI;\n"),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
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
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

pub(super) fn create_sample_toml_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(&repo_dir)?;
    fs::write(
        repo_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            package_name.to_ascii_lowercase()
        ),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

pub(super) async fn load_code_search_response(
    studio: &crate::studio::StudioState,
    query: &str,
) -> crate::contracts::SearchResponse {
    build_code_search_response(studio, query.to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("code search response for `{query}`: {error:?}"))
}

pub(super) fn create_sample_html_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(&repo_dir)?;
    fs::write(
        repo_dir.join("index.html"),
        format!(
            "<!doctype html>\n<html>\n  <head>\n    <title>{package_name}</title>\n  </head>\n  <body>\n    <main><section>search fixture</section></main>\n  </body>\n</html>\n"
        ),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

fn initialize_git_fixture(repo_dir: &Path, package_name: &str) {
    init_git_repository(repo_dir);
    let remote_url = format!(
        "https://example.invalid/xiuxian-wendao/{}.git",
        package_name.to_ascii_lowercase()
    );
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["remote", "add", "origin", remote_url.as_str()])
        .output()
        .unwrap_or_else(|error| panic!("add git remote: {error}"));
    assert!(
        output.status.success(),
        "add git remote failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    commit_all(repo_dir, "initial import");
}
