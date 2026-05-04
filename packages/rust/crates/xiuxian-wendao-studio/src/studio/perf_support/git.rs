use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};

const GIT_AUTHOR_NAME: &str = "Gateway Perf";
const GIT_AUTHOR_EMAIL: &str = "gateway-perf@example.invalid";

#[cfg(not(feature = "julia"))]
pub(crate) fn write_default_repo_config(base: &Path, repo_dir: &Path, repo_id: &str) -> Result<()> {
    write_repo_config(base, repo_dir, repo_id, None)
}

#[cfg(feature = "julia")]
pub(crate) fn write_repo_config_with_julia_parser_summary_transport(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
    base_url: &str,
) -> Result<()> {
    write_repo_config(base, repo_dir, repo_id, Some(base_url))
}

fn write_repo_config(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
    parser_summary_base_url: Option<&str>,
) -> Result<()> {
    let plugin = parser_summary_base_url.map_or_else(
        || "\"julia\"".to_string(),
        |base_url| {
            format!(
                "{{ id = \"julia\", parser_summary_transport = {{ base_url = \"{base_url}\", file_summary = {{ schema_version = \"v3\" }}, root_summary = {{ schema_version = \"v3\" }} }} }}"
            )
        },
    );
    fs::write(
        base.join("wendao.toml"),
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = [
  {plugin}
]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

pub(crate) fn create_local_git_repo(base: &Path, package_name: &str) -> Result<PathBuf> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(repo_dir.join("README.md"), "# Gateway Repo\n")?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            r#"name = "{package_name}"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"
"#
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!(
            "module {package_name}\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n"
        ),
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        format!("using {package_name}\nsolve()\n"),
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;

    let repo_dir_arg = repo_dir.display().to_string();
    run_git(None, &["init", "--quiet", repo_dir_arg.as_str()])?;
    let remote_url = format!(
        "https://example.invalid/xiuxian-wendao/{}.git",
        package_name.to_ascii_lowercase()
    );
    run_git(
        Some(&repo_dir),
        &["remote", "add", "origin", remote_url.as_str()],
    )?;
    commit_all(&repo_dir, "initial import")?;
    Ok(repo_dir)
}

fn commit_all(repo_dir: &Path, message: &str) -> Result<()> {
    run_git(Some(repo_dir), &["add", "."])?;
    run_git(Some(repo_dir), &["commit", "--quiet", "-m", message])
}

fn run_git(cwd: Option<&Path>, args: &[&str]) -> Result<()> {
    let mut command = Command::new("git");
    if let Some(cwd) = cwd {
        command.arg("-C").arg(cwd);
    }
    command
        .args(args)
        .env("GIT_AUTHOR_NAME", GIT_AUTHOR_NAME)
        .env("GIT_AUTHOR_EMAIL", GIT_AUTHOR_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_AUTHOR_NAME)
        .env("GIT_COMMITTER_EMAIL", GIT_AUTHOR_EMAIL);
    let output = command
        .output()
        .with_context(|| format!("failed to spawn git `{}`", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{stderr}; stdout: {stdout}")
    };
    Err(anyhow!(
        "git {} failed: {}",
        args.join(" "),
        if detail.is_empty() {
            "unknown error".to_string()
        } else {
            detail
        }
    ))
}
