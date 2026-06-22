use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::{
    advance_workdir_step, check_flowhub, check_flowhub_scenario, check_workdir,
    classify_flowhub_dir, looks_like_flowhub_scenario_dir, looks_like_workdir_dir,
    render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    render_workdir_advance, render_workdir_check_markdown, render_workdir_show, show_flowhub,
    show_flowhub_anchored_scenario, show_flowhub_graph, show_flowhub_scenario, show_workdir,
};
#[cfg(feature = "wendao-integration")]
use crate::{render_wendao_docs_contract_show, show_wendao_docs_contract};

use super::output::{render_missing_workdir_root_output, render_uninitialized_workdir_root_output};
use super::types::{DirCliCommand, DirCliOutput, ShowCliTarget};
use crate::qianji_cli::workspace::resolve_workspace_root;

pub(crate) fn handle_dir_command(command: DirCliCommand) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_dir_command(command)?;
    println!("{}", output.rendered);
    if output.exit_code == 0 {
        Ok(())
    } else {
        std::process::exit(output.exit_code);
    }
}

pub(crate) fn run_dir_command(command: DirCliCommand) -> Result<DirCliOutput, QianjiError> {
    match command {
        DirCliCommand::Show { target } => run_show_command(&target),
        DirCliCommand::Check { dir } => run_check_dir_command(&dir),
        DirCliCommand::Advance { dir, to } => run_advance_command(&dir, &to),
    }
}

fn run_show_command(target: &ShowCliTarget) -> Result<DirCliOutput, QianjiError> {
    match target {
        ShowCliTarget::Dir(dir) => run_show_dir_command(dir),
        ShowCliTarget::Graph(graph) => run_show_graph_command(graph),
        ShowCliTarget::Contract(contract_name) => run_show_contract_command(contract_name),
        ShowCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
        } => run_show_anchored_scenario_command(anchor, scenario, dir.as_deref()),
    }
}

fn run_show_dir_command(dir: &Path) -> Result<DirCliOutput, QianjiError> {
    if looks_like_workdir_dir(dir)? {
        let show = show_workdir(dir)?;
        return Ok(DirCliOutput {
            rendered: render_workdir_show(&show),
            exit_code: 0,
        });
    }

    if classify_flowhub_dir(dir)?.is_some() {
        let show = show_flowhub(dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_show(&show),
            exit_code: 0,
        });
    }

    if looks_like_flowhub_scenario_dir(dir) {
        let flowhub_root = resolve_flowhub_root_for_scenario_dir(dir).map_err(|error| {
            QianjiError::Topology(format!(
                "failed to resolve default Flowhub root for scenario `{}`: {error}",
                dir.display()
            ))
        })?;
        let show = show_flowhub_scenario(&flowhub_root, dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_scenario_show(&show),
            exit_code: 0,
        });
    }

    Err(QianjiError::Topology(format!(
        "`{}` is neither a bounded work surface, a Flowhub root/module, nor a Flowhub scenario directory",
        dir.display()
    )))
}

fn run_show_graph_command(graph: &Path) -> Result<DirCliOutput, QianjiError> {
    let show = show_flowhub_graph(graph)?;
    Ok(DirCliOutput {
        rendered: render_flowhub_graph_show(&show),
        exit_code: 0,
    })
}

fn run_show_contract_command(contract_name: &str) -> Result<DirCliOutput, QianjiError> {
    #[cfg(not(feature = "wendao-integration"))]
    {
        let _ = contract_name;
        Err(QianjiError::Topology(
            "`show --contract` requires the `wendao-integration` feature".to_string(),
        ))
    }
    #[cfg(feature = "wendao-integration")]
    {
        let show = show_wendao_docs_contract(contract_name)?;
        Ok(DirCliOutput {
            rendered: render_wendao_docs_contract_show(&show),
            exit_code: 0,
        })
    }
}

fn run_show_anchored_scenario_command(
    anchor: &Path,
    scenario: &str,
    dir: Option<&Path>,
) -> Result<DirCliOutput, QianjiError> {
    Ok(DirCliOutput {
        rendered: show_flowhub_anchored_scenario(anchor, scenario, dir)?,
        exit_code: 0,
    })
}

fn run_check_dir_command(dir: &Path) -> Result<DirCliOutput, QianjiError> {
    if looks_like_workdir_dir(dir)? {
        let report = check_workdir(dir)?;
        return Ok(DirCliOutput {
            rendered: render_workdir_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if classify_flowhub_dir(dir)?.is_some() {
        let report = check_flowhub(dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if looks_like_flowhub_scenario_dir(dir) {
        let flowhub_root = resolve_flowhub_root_for_scenario_dir(dir).map_err(|error| {
            QianjiError::Topology(format!(
                "failed to resolve default Flowhub root for scenario `{}`: {error}",
                dir.display()
            ))
        })?;
        let report = check_flowhub_scenario(&flowhub_root, dir);
        return Ok(DirCliOutput {
            rendered: render_flowhub_scenario_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if !dir.exists() {
        return Ok(render_missing_workdir_root_output(dir));
    }

    if dir.is_dir() {
        return Ok(render_uninitialized_workdir_root_output(dir));
    }

    Err(QianjiError::Topology(format!(
        "`{}` is neither a bounded work surface, a Flowhub root/module, nor a Flowhub scenario directory",
        dir.display()
    )))
}

fn run_advance_command(dir: &Path, to: &str) -> Result<DirCliOutput, QianjiError> {
    let advance = advance_workdir_step(dir, to)?;
    Ok(DirCliOutput {
        rendered: render_workdir_advance(&advance),
        exit_code: 0,
    })
}

fn resolve_default_flowhub_root() -> std::io::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    if let Some(root) = find_default_flowhub_root(current_dir.as_path()) {
        return Ok(root);
    }
    if let Some(root) = find_default_flowhub_root(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        return Ok(root);
    }
    Ok(resolve_workspace_root(None)?.join("qianji-flowhub"))
}

fn resolve_flowhub_root_for_scenario_dir(dir: &Path) -> std::io::Result<PathBuf> {
    if let Some(root) = find_default_flowhub_root(dir) {
        return Ok(root);
    }
    resolve_default_flowhub_root()
}

fn find_default_flowhub_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|ancestor| ancestor.join("qianji-flowhub"))
        .find(|candidate| candidate.join("qianji.toml").is_file())
}
