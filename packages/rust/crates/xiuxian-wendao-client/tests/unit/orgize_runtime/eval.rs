use std::path::Path;

use super::support::tempdir_or_panic;

#[test]
fn standalone_orgize_eval_plan_and_patch_use_host_supplied_output() {
    let temp = tempdir_or_panic();
    let org_path = temp.path().join("task.org");
    std::fs::write(&org_path, eval_fixture())
        .unwrap_or_else(|error| panic!("write eval fixture: {error}"));

    let plan = run_orgize(
        temp.path(),
        &["eval", "plan", "verify", "task.org"],
        "eval plan",
    );
    assert_eq!(
        plan.status_code,
        Some(0),
        "stdout: {}\nstderr: {}",
        plan.stdout,
        plan.stderr
    );
    assert!(plan.stdout.contains("name: verify"));
    assert!(plan.stdout.contains("results: output replace"));
    assert!(!plan.stdout.contains("source:"));

    let patch = run_orgize(
        temp.path(),
        &[
            "eval",
            "patch",
            "--write",
            "--stdout",
            "ok",
            "--exit-code",
            "0",
            "verify",
            "task.org",
        ],
        "eval patch",
    );
    assert_eq!(
        patch.status_code,
        Some(0),
        "stdout: {}\nstderr: {}",
        patch.stdout,
        patch.stderr
    );
    assert!(patch.stdout.contains("kind: insert"));
    assert!(patch.stdout.contains("written: true"));
    assert_eq!(
        std::fs::read_to_string(org_path)
            .unwrap_or_else(|error| panic!("read patched org: {error}")),
        concat!(
            "#+NAME: verify\n",
            "#+BEGIN_SRC bash :results output replace\n",
            "echo should-not-run\n",
            "#+END_SRC\n",
            "\n",
            "#+RESULTS: verify\n",
            ": ok\n",
        )
    );
}

#[test]
fn standalone_orgize_eval_patch_resolves_output_files_from_client_root() {
    let temp = tempdir_or_panic();
    let org_path = temp.path().join("task.org");
    let stdout_path = temp.path().join("stdout.txt");
    std::fs::write(&org_path, eval_fixture())
        .unwrap_or_else(|error| panic!("write eval fixture: {error}"));
    std::fs::write(&stdout_path, "root output")
        .unwrap_or_else(|error| panic!("write stdout fixture: {error}"));

    let patch = run_orgize(
        temp.path(),
        &[
            "eval",
            "patch",
            "--write",
            "--stdout-file",
            "stdout.txt",
            "verify",
            "task.org",
        ],
        "eval patch from stdout file",
    );
    assert_eq!(
        patch.status_code,
        Some(0),
        "stdout: {}\nstderr: {}",
        patch.stdout,
        patch.stderr
    );
    assert!(patch.stdout.contains("written: true"));
    assert!(
        std::fs::read_to_string(org_path)
            .unwrap_or_else(|error| panic!("read patched org: {error}"))
            .contains(": root output")
    );
}

fn run_orgize(root: &Path, args: &[&str], context: &str) -> CliOutput {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(root)
        .arg("orgize")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run orgize {context}: {error}"));
    CliOutput {
        stdout: String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("stdout utf8: {error}")),
        stderr: String::from_utf8(output.stderr)
            .unwrap_or_else(|error| panic!("stderr utf8: {error}")),
        status_code: output.status.code(),
    }
}

struct CliOutput {
    stdout: String,
    stderr: String,
    status_code: Option<i32>,
}

fn eval_fixture() -> &'static str {
    concat!(
        "#+NAME: verify\n",
        "#+BEGIN_SRC bash :results output replace\n",
        "echo should-not-run\n",
        "#+END_SRC\n",
    )
}
