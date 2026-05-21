#[cfg(feature = "orgize-agent-read-model")]
pub(crate) struct CliOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) status_code: Option<i32>,
}

#[cfg(feature = "orgize-agent-read-model")]
pub(crate) fn run_orgize(root: &std::path::Path, args: &[&str], context: &str) -> CliOutput {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(root)
        .arg("orgize")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run orgize {context}: {error}"));
    cli_output(output)
}

#[cfg(feature = "orgize-agent-read-model")]
fn cli_output(output: std::process::Output) -> CliOutput {
    CliOutput {
        stdout: String::from_utf8(output.stdout)
            .unwrap_or_else(|error| panic!("stdout utf8: {error}")),
        stderr: String::from_utf8(output.stderr)
            .unwrap_or_else(|error| panic!("stderr utf8: {error}")),
        status_code: output.status.code(),
    }
}

#[cfg(feature = "orgize-agent-read-model")]
pub(crate) fn assert_cli_success(output: &CliOutput) {
    assert_eq!(
        output.status_code,
        Some(0),
        "stdout: {}\nstderr: {}",
        output.stdout,
        output.stderr
    );
}

pub(crate) fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}
