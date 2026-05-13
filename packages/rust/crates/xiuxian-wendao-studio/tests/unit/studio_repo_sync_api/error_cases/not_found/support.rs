use std::path::{Path, PathBuf};

use crate::studio::studio_repo_sync_api_tests::support::{
    create_local_git_repo, write_default_repo_config,
};
use crate::studio::studio_repo_sync_api_tests::{
    ProjectionPageKind, RepoProjectedPagesQuery, TestResult, fs, repo_projected_pages_from_config,
};

pub(crate) fn prepare_gateway_sync_repo(root: &Path) -> TestResult<PathBuf> {
    let repo_dir = create_local_git_repo(root, "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(root, &repo_dir, "gateway-sync")?;
    Ok(repo_dir)
}

pub(crate) fn gateway_sync_symbol_reference_page_id(root: &Path) -> TestResult<String> {
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        root,
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    Ok(page.page_id.clone())
}
