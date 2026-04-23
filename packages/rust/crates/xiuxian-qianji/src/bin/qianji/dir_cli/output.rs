use std::path::Path;

use xiuxian_qianji::{WorkdirCheckReport, WorkdirDiagnostic, render_workdir_check_markdown};

use super::types::DirCliOutput;

pub(super) fn render_missing_workdir_root_output(dir: &Path) -> DirCliOutput {
    render_workdir_bootstrap_output(
        dir,
        "Missing workdir root",
        format!(
            "localized run root `{}` does not exist, so `qianji check` has no bounded work surface to evaluate",
            dir.display()
        ),
        format!(
            "create `{}` with `qianji.toml`, `flowchart.mmd`, and the scenario-declared runtime surfaces before rerunning `qianji check --dir {}`; use `qianji show --anchor <module-qianji.toml> --scenario <name> --dir {}` to inspect the expected surface first",
            dir.display(),
            dir.display(),
            dir.display()
        ),
    )
}

pub(super) fn render_uninitialized_workdir_root_output(dir: &Path) -> DirCliOutput {
    render_workdir_bootstrap_output(
        dir,
        "Uninitialized workdir root",
        format!(
            "`{}` exists, but it does not yet declare a bounded work-surface manifest (`qianji.toml` with `[plan]` and `[check]`)",
            dir.display()
        ),
        format!(
            "initialize `{}` with `qianji.toml`, `flowchart.mmd`, and the scenario-declared runtime surfaces before rerunning `qianji check --dir {}`; use `qianji show --anchor <module-qianji.toml> --scenario <name> --dir {}` to inspect the expected surface first",
            dir.display(),
            dir.display(),
            dir.display()
        ),
    )
}

fn render_workdir_bootstrap_output(
    dir: &Path,
    title: &str,
    problem: String,
    fix: String,
) -> DirCliOutput {
    let report = WorkdirCheckReport {
        plan_name: "uninitialized-workdir".to_string(),
        workdir: dir.to_path_buf(),
        diagnostics: vec![WorkdirDiagnostic {
            title: title.to_string(),
            location: dir.to_path_buf(),
            problem,
            why_it_blocks:
                "Qianji cannot evaluate localized step boundaries until the run root is materialized"
                    .to_string(),
            fix,
            follow_up_surfaces: Vec::new(),
        }],
    };

    DirCliOutput {
        rendered: render_workdir_check_markdown(&report),
        exit_code: 2,
    }
}
