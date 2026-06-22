use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_qianji_client::{
    FlowhubCliOutput, QianjiClientError, run_xiuxian_qianji_client_cli_with_args,
};

pub(super) struct FlowhubTestProject {
    _temp_dir: TempDir,
    pub(super) project_root: PathBuf,
    pub(super) cache_home: PathBuf,
    pub(super) flowhub_root: PathBuf,
}

impl FlowhubTestProject {
    pub(super) fn live() -> Self {
        Self::new(true)
    }

    pub(super) fn isolated_flowhub() -> Self {
        Self::new(false)
    }

    fn new(write_default_fixture: bool) -> Self {
        let temp_dir =
            TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
        let project_root = temp_dir.path().join("downstream");
        let cache_home = project_root.join(".cache");
        let flowhub_root = temp_dir.path().join("flowhub");
        if write_default_fixture {
            write_source_pair(&flowhub_root, "plan", "agent-coding", "agent_coding");
            write_source_pair(
                &flowhub_root,
                "research/paper",
                "deep_read",
                "paper_deep_read",
            );
            write_source_pair(
                &flowhub_root,
                "wendao",
                "wendao-client-plan-policy",
                "wendao_client_plan_policy",
            );
        }
        Self {
            _temp_dir: temp_dir,
            project_root,
            cache_home,
            flowhub_root,
        }
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

    pub(super) fn lint_args(
        &self,
        scenario: Option<&str>,
        slug: Option<&str>,
        json: bool,
    ) -> Vec<String> {
        let mut head = vec!["lint"];
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

    pub(super) fn lint_all_args(&self, json: bool) -> Vec<String> {
        let mut args = self.flowhub_args(&["lint", "--all"]);
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

    pub(super) fn flowhub_args(&self, action_args: &[&str]) -> Vec<String> {
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

pub(super) fn run(args: &[String], label: &str) -> FlowhubCliOutput {
    run_xiuxian_qianji_client_cli_with_args(args)
        .unwrap_or_else(|error| panic!("{label} should render a report: {error}"))
}

pub(super) fn run_error(args: &[String], label: &str) -> String {
    match run_xiuxian_qianji_client_cli_with_args(args) {
        Ok(output) => panic!("{label} should fail, rendered output: {}", output.rendered),
        Err(QianjiClientError::Message(message)) => message,
    }
}

pub(super) fn rendered_json(output: &FlowhubCliOutput) -> serde_json::Value {
    serde_json::from_str(&output.rendered)
        .unwrap_or_else(|error| panic!("rendered output should be JSON: {error}"))
}

pub(super) fn copy_agent_coding_pair(project: &FlowhubTestProject, relative_dir: &str) {
    write_source_pair(
        &project.flowhub_root,
        relative_dir,
        "agent-coding",
        "agent_coding",
    );
}

fn write_source_pair(flowhub_root: &Path, relative_dir: &str, scenario: &str, process_id: &str) {
    let source_root = flowhub_root.join(relative_dir);
    std::fs::create_dir_all(&source_root)
        .unwrap_or_else(|error| panic!("Flowhub source root should be created: {error}"));
    let stem = match scenario {
        "deep_read" => "paper-deep-read",
        "wendao-client-plan-policy" => "wendao-client-plan-policy",
        _ => scenario,
    };
    let org_name = format!("{stem}.org");
    let bpmn_name = format!("{stem}.bpmn");
    std::fs::write(
        source_root.join(&org_name),
        minimal_flowhub_org(scenario, process_id, &bpmn_name),
    )
    .unwrap_or_else(|error| panic!("Org source should write: {error}"));
    std::fs::write(source_root.join(&bpmn_name), minimal_bpmn(process_id))
        .unwrap_or_else(|error| panic!("BPMN source should write: {error}"));
}

fn minimal_flowhub_org(scenario: &str, process_id: &str, bpmn_name: &str) -> String {
    format!(
        r#"#+TITLE: {scenario} Flowhub Source

* Scenario
:PROPERTIES:
:FLOWHUB_SCENARIO_ID: {scenario}
:CANONICAL_SOURCE: org+bpmn
:BPMN_SOURCE: {bpmn_name}
:BPMN_PROCESS_ID: {process_id}
:END:

#+begin_src mermaid
flowchart LR
  Start["Start"] --> Done["Done"]
#+end_src
"#
    )
}

fn minimal_bpmn(process_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_{process_id}" targetNamespace="https://example.test/qianji-client-flowhub">
  <bpmn:process id="{process_id}" isExecutable="true">
    <bpmn:startEvent id="start">
      <bpmn:outgoing>flow_start_done</bpmn:outgoing>
    </bpmn:startEvent>
    <bpmn:endEvent id="done">
      <bpmn:incoming>flow_start_done</bpmn:incoming>
    </bpmn:endEvent>
    <bpmn:sequenceFlow id="flow_start_done" sourceRef="start" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>
"#
    )
}
