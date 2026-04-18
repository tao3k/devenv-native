use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub use super::linked_parser_summary::{
    ensure_linked_julia_parser_summary_service, ensure_linked_modelica_parser_summary_service,
};

pub type TestResult = Result<(), Box<dyn std::error::Error>>;
pub type TestResultPath = Result<PathBuf, Box<dyn std::error::Error>>;

const TEST_GIT_AUTHOR_NAME: &str = "Xiuxian Test";
const TEST_GIT_AUTHOR_EMAIL: &str = "test@example.com";
const TEST_GIT_COMMIT_TIME: &str = "1700000000 +0000";

pub fn create_sample_julia_repo(
    base: &Path,
    package_name: &str,
    expected_root: bool,
) -> TestResultPath {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::create_dir_all(repo_dir.join("src").join("nested"))?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::create_dir_all(repo_dir.join("test"))?;
    fs::create_dir_all(repo_dir.join("docs"))?;

    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            r#"name = "{package_name}"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"

[deps]
SciMLBase = "0bca4576-84f4-4d90-8ffe-ffa030f20462"
LinearAlgebra = "37e2e46d-f89d-539d-b4ee-838fcccc9c8e"
"#
        ),
    )?;

    let module_name = if expected_root {
        package_name.to_string()
    } else {
        format!("{package_name}Alt")
    };
    let root_file_name = if expected_root {
        format!("{package_name}.jl")
    } else {
        "Other.jl".to_string()
    };
    fs::write(
        repo_dir.join("src").join(root_file_name),
        format!(
            r#"module {module_name}

export solve, Problem
using LinearAlgebra
@reexport using SciMLBase
include("solvers.jl")

"""
Problem docs.
"""
struct Problem
    x::Int
end

"""
Solve docs.
"""
function solve(problem::Problem)
    problem.x
end

end
"#
        ),
    )?;
    fs::write(
        repo_dir.join("src").join("solvers.jl"),
        r#"
"""
Fast solve docs.
"""
fastsolve(problem::Problem) = problem.x

include("nested/extra.jl")
"#,
    )?;
    fs::write(
        repo_dir.join("src").join("nested").join("extra.jl"),
        r#"
"""
Extra problem docs.
"""
struct ExtraProblem
    y::Int
end
"#,
    )?;

    fs::write(
        repo_dir.join("examples").join("basic.jl"),
        "problem = Problem(1)\nsolve(problem)\nfastsolve(problem)\n",
    )?;
    fs::write(
        repo_dir.join("test").join("runtests.jl"),
        "extra = ExtraProblem(2)\nprintln(extra)\n",
    )?;
    fs::write(repo_dir.join("README.md"), "# Sample\n")?;
    fs::write(repo_dir.join("docs").join("guide.md"), "# Guide\n")?;
    initialize_git_repository(
        &repo_dir,
        &format!(
            "https://example.invalid/{}/{}.git",
            "xiuxian-wendao",
            package_name.to_ascii_lowercase()
        ),
    )?;
    Ok(repo_dir)
}

pub fn create_sample_modelica_repo(base: &Path, package_name: &str) -> TestResultPath {
    ensure_linked_modelica_parser_summary_service()?;
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers").join("Examples"))?;
    fs::create_dir_all(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial"),
    )?;

    fs::write(repo_dir.join("README.md"), format!("# {package_name}\n"))?;
    fs::write(repo_dir.join("package.order"), "Controllers\n")?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            "within;\npackage {package_name}\n  annotation(Documentation(info = \"<html>{package_name} package docs.</html>\"));\nend {package_name};\n",
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("package.mo"),
        format!("within {package_name};\npackage Controllers\nend Controllers;\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("PI.mo"),
        format!(
            "within {package_name}.Controllers;\nmodel PI\n  annotation(Documentation(info = \"<html>PI controller docs.</html>\"));\nend PI;\n",
        ),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("package.order"),
        "Step\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("Step.mo"),
        format!("within {package_name}.Controllers.Examples;\nmodel Step\nend Step;\n"),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.order"),
        "Tutorial\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.mo"),
        format!("within {package_name}.Controllers;\npackage UsersGuide\nend UsersGuide;\n",),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial")
            .join("FirstSteps.mo"),
        format!(
            "within {package_name}.Controllers.UsersGuide.Tutorial;\nmodel FirstSteps\n  annotation(Documentation(info = \"<html>First steps guide.</html>\"));\nend FirstSteps;\n",
        ),
    )?;

    initialize_git_repository(
        &repo_dir,
        &format!(
            "https://example.invalid/{}/{}.git",
            "xiuxian-wendao",
            package_name.to_ascii_lowercase()
        ),
    )?;
    Ok(repo_dir)
}

pub fn initialize_git_repository(repo_dir: &Path, remote_url: &str) -> TestResult {
    let repo_dir_arg = repo_dir.display().to_string();
    run_git(None, &["init", "--quiet", repo_dir_arg.as_str()])?;
    run_git(Some(repo_dir), &["remote", "add", "origin", remote_url])?;
    commit_all(repo_dir, "initial import")?;
    ensure_branch_main(repo_dir)?;
    Ok(())
}

pub fn append_repo_file_and_commit(
    repo_dir: &Path,
    relative_path: &str,
    contents: &str,
    message: &str,
) -> TestResult {
    let target = repo_dir.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, contents)?;
    commit_all(repo_dir, message)?;
    Ok(())
}

pub fn commit_all(repo_dir: &Path, message: &str) -> TestResult {
    run_git(Some(repo_dir), &["add", "--all"])?;
    run_git(Some(repo_dir), &["commit", "--quiet", "-m", message])?;
    Ok(())
}

pub fn ensure_branch_main(repo_dir: &Path) -> TestResult {
    run_git(Some(repo_dir), &["branch", "-M", "main"])?;
    Ok(())
}

pub fn refresh_remote(repo_dir: &Path, remote_name: &str) -> TestResult {
    run_git(
        Some(repo_dir),
        &[
            "fetch",
            remote_name,
            "+refs/heads/*:refs/heads/*",
            "+refs/tags/*:refs/tags/*",
        ],
    )?;
    Ok(())
}

pub fn git_remote_url(
    repo_dir: &Path,
    remote_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    run_git(
        Some(repo_dir),
        &["config", "--get", &format!("remote.{remote_name}.url")],
    )
}

pub fn git_is_bare_repository(repo_dir: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(run_git(Some(repo_dir), &["rev-parse", "--is-bare-repository"])? == "true")
}

fn run_git(cwd: Option<&Path>, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = Command::new("git");
    if let Some(cwd) = cwd {
        command.arg("-C").arg(cwd);
    }
    let output = command
        .args(args)
        .env("GIT_AUTHOR_NAME", TEST_GIT_AUTHOR_NAME)
        .env("GIT_AUTHOR_EMAIL", TEST_GIT_AUTHOR_EMAIL)
        .env("GIT_COMMITTER_NAME", TEST_GIT_AUTHOR_NAME)
        .env("GIT_COMMITTER_EMAIL", TEST_GIT_AUTHOR_EMAIL)
        .env("GIT_AUTHOR_DATE", TEST_GIT_COMMIT_TIME)
        .env("GIT_COMMITTER_DATE", TEST_GIT_COMMIT_TIME)
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = match (stderr.is_empty(), stdout.is_empty()) {
        (false, true) => stderr,
        (true, false) => stdout,
        (false, false) => format!("{stderr}; stdout: {stdout}"),
        (true, true) => "unknown git error".to_string(),
    };
    Err(format!("git {} failed: {detail}", args.join(" ")).into())
}
