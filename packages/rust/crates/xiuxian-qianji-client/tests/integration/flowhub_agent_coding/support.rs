use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_qianji_client::{FlowhubCliOutput, run_xiuxian_qianji_client_cli_with_args};

pub(super) struct FlowhubTestProject {
    _temp_dir: TempDir,
    pub(super) project_root: PathBuf,
    pub(super) cache_home: PathBuf,
    pub(super) flowhub_root: PathBuf,
}

impl FlowhubTestProject {
    pub(super) fn live() -> Self {
        let temp_dir =
            TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
        let project_root = temp_dir.path().join("downstream");
        let cache_home = project_root.join(".cache");
        let flowhub_root = repo_root().join("qianji-flowhub");
        Self {
            _temp_dir: temp_dir,
            project_root,
            cache_home,
            flowhub_root,
        }
    }

    pub(super) fn isolated_flowhub() -> Self {
        let mut project = Self::live();
        project.flowhub_root = project
            .project_root
            .parent()
            .unwrap_or_else(|| panic!("project root should have a parent"))
            .join("flowhub");
        project
    }

    pub(super) fn init_args(&self, scenario: &str, slug: Option<&str>, json: bool) -> Vec<String> {
        let mut args = self.flowhub_args(&["--mode", "plan", "--scenario", scenario, "init"]);
        if let Some(slug) = slug {
            args.insert(7, "--slug".to_string());
            args.insert(8, slug.to_string());
        }
        if json {
            args.push("--json".to_string());
        }
        args
    }

    pub(super) fn check_args(
        &self,
        scenario: Option<&str>,
        slug: Option<&str>,
        json: bool,
    ) -> Vec<String> {
        let mut head = vec!["check"];
        if let Some(scenario) = scenario {
            head.extend(["--scenario", scenario]);
        }
        if let Some(slug) = slug {
            head.extend(["--slug", slug]);
        }
        let mut args = self.flowhub_args(&head);
        if json {
            args.push("--json".to_string());
        }
        args
    }

    pub(super) fn check_all_args(&self, json: bool) -> Vec<String> {
        let mut args = self.flowhub_args(&["check", "--all"]);
        if json {
            args.push("--json".to_string());
        }
        args
    }

    pub(super) fn scenarios_args(&self, json: bool) -> Vec<String> {
        let mut args = self.flowhub_args(&["scenarios"]);
        if json {
            args.push("--json".to_string());
        }
        args
    }

    fn flowhub_args(&self, action_args: &[&str]) -> Vec<String> {
        let mut args = vec!["qianji-client".to_string(), "flowhub".to_string()];
        args.extend(action_args.iter().map(|arg| (*arg).to_string()));
        args.extend([
            "--project-root".to_string(),
            self.project_root.display().to_string(),
            "--cache-home".to_string(),
            self.cache_home.display().to_string(),
            "--flowhub-root".to_string(),
            self.flowhub_root.display().to_string(),
        ]);
        args
    }
}

pub(super) fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.ancestors().nth(4) {
        Some(root) => root.to_path_buf(),
        None => panic!("workspace root should be four ancestors above qianji-client crate"),
    }
}

pub(super) fn run(args: &[String], label: &str) -> FlowhubCliOutput {
    run_xiuxian_qianji_client_cli_with_args(args)
        .unwrap_or_else(|error| panic!("{label} should render a report: {error}"))
}

pub(super) fn rendered_json(output: &FlowhubCliOutput) -> serde_json::Value {
    serde_json::from_str(&output.rendered)
        .unwrap_or_else(|error| panic!("rendered output should be JSON: {error}"))
}

pub(super) fn copy_agent_coding_pair(project: &FlowhubTestProject, relative_dir: &str) {
    let source_root = project.flowhub_root.join(relative_dir);
    std::fs::create_dir_all(&source_root)
        .unwrap_or_else(|error| panic!("Flowhub source root should be created: {error}"));
    std::fs::copy(
        repo_root().join("qianji-flowhub/plan/agent-coding.org"),
        source_root.join("agent-coding.org"),
    )
    .unwrap_or_else(|error| panic!("Org source should copy: {error}"));
    std::fs::copy(
        repo_root().join("qianji-flowhub/plan/agent-coding.bpmn"),
        source_root.join("agent-coding.bpmn"),
    )
    .unwrap_or_else(|error| panic!("BPMN source should copy: {error}"));
}
