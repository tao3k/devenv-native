use super::deps::{
    BTreeMap, Deserialize, Path, PathBuf, QianjiBpmnCheckpointStore, QianjiBpmnHostBridge,
    QianjiBpmnWorkflowCheckpointBackend, empty_json_object,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BpmnCliCommand {
    Start(BpmnStartCliCommand),
    StartAt(BpmnStartAtCliCommand),
    Run(BpmnRunCliCommand),
    HostSession(BpmnHostSessionCliCommand),
    Resume(BpmnResumeCliCommand),
    EventPoll(BpmnEventPollCliCommand),
    TaskComplete(BpmnTaskCompleteCliCommand),
    TaskClaim(BpmnTaskClaimCliCommand),
    TaskRelease(BpmnTaskReleaseCliCommand),
    TaskWorklist(BpmnTaskWorklistCliCommand),
    Status(BpmnStatusCliCommand),
    Instances(BpmnInstancesCliCommand),
    Cancel(BpmnCancelCliCommand),
    Interrupt(BpmnInterruptCliCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnRunCliCommand {
    pub(crate) bpmn_path: PathBuf,
    pub(crate) dmn_paths: Vec<PathBuf>,
    pub(crate) process_id: String,
    pub(crate) instance_id: String,
    pub(crate) context_json: Option<String>,
    pub(crate) start_at_node_id: Option<String>,
    pub(crate) checkpoint_backend: Option<QianjiBpmnWorkflowCheckpointBackend>,
    pub(crate) host_fixture_path: Option<PathBuf>,
    pub(crate) event_fixture_path: Option<PathBuf>,
    pub(crate) trace_stream: bool,
    pub(crate) external_host: bool,
    pub(crate) continue_until_human_boundary: bool,
}

pub(crate) type BpmnStartCliCommand = BpmnRunCliCommand;

pub(crate) type BpmnStartAtCliCommand = BpmnRunCliCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnHostSessionCliCommand {
    pub(crate) start: BpmnRunCliCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnResumeCliCommand {
    pub(crate) bpmn_path: PathBuf,
    pub(crate) dmn_paths: Vec<PathBuf>,
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) host_fixture_path: Option<PathBuf>,
    pub(crate) event_fixture_path: Option<PathBuf>,
    pub(crate) trace_stream: bool,
    pub(crate) external_host: bool,
    pub(crate) continue_until_human_boundary: bool,
}

pub(crate) type BpmnEventPollCliCommand = BpmnResumeCliCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnTaskCompleteCliCommand {
    pub(crate) bpmn_path: PathBuf,
    pub(crate) dmn_paths: Vec<PathBuf>,
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) token_id: u64,
    pub(crate) process_id: String,
    pub(crate) activity_id: String,
    pub(crate) kind: BpmnTaskCompleteCliKind,
    pub(crate) data_json: String,
    pub(crate) claimant: Option<String>,
    pub(crate) host_fixture_path: Option<PathBuf>,
    pub(crate) event_fixture_path: Option<PathBuf>,
    pub(crate) trace_stream: bool,
    pub(crate) continue_until_human_boundary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnTaskClaimCliCommand {
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) token_id: u64,
    pub(crate) process_id: String,
    pub(crate) activity_id: String,
    pub(crate) claimant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnTaskReleaseCliCommand {
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) token_id: u64,
    pub(crate) process_id: String,
    pub(crate) activity_id: String,
    pub(crate) claimant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnTaskWorklistCliCommand {
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) claimant: Option<String>,
    pub(crate) assignment_resource: Option<String>,
    pub(crate) lane: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BpmnTaskCompleteCliKind {
    Task,
    Send,
    Service,
    Script,
    User,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnStatusCliCommand {
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    pub(crate) bpmn_path: Option<PathBuf>,
    pub(crate) dmn_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnInstancesCliCommand {
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnCancelCliCommand {
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnInterruptCliCommand {
    pub(crate) instance_id: String,
    pub(crate) checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}

pub(crate) struct BpmnExecutionRenderContext<'a> {
    pub(crate) resolved_bpmn_path: &'a Path,
    pub(crate) resolved_dmn_paths: &'a [PathBuf],
    pub(crate) checkpoint_store: Option<&'a QianjiBpmnCheckpointStore>,
    pub(crate) resolved_host_fixture_path: Option<&'a Path>,
    pub(crate) resolved_event_fixture_path: Option<&'a Path>,
    pub(crate) resumed_from_checkpoint: bool,
    pub(crate) checkpoint_saved: bool,
    pub(crate) checkpoint_deleted: bool,
}

pub(crate) struct BpmnCliHostBridgeContext {
    pub(crate) host: QianjiBpmnHostBridge,
    pub(crate) resolved_host_fixture_path: Option<PathBuf>,
    pub(crate) resolved_event_fixture_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct BpmnCliHostFixture {
    #[serde(rename = "tasks")]
    pub(crate) task: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "send_tasks")]
    pub(crate) send: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "service_tasks")]
    pub(crate) service: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "service_task_tokens")]
    pub(crate) service_by_token: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "user_tasks")]
    pub(crate) user: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "manual_tasks")]
    pub(crate) manual: BTreeMap<String, BpmnCliHostDataFixture>,
    #[serde(rename = "business_rule_tasks")]
    pub(crate) business_rule: BTreeMap<String, BpmnCliBusinessRuleFixture>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BpmnCliHostDataFixture {
    pub(crate) data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BpmnCliBusinessRuleFixture {
    pub(crate) output: serde_json::Value,
    #[serde(default)]
    pub(crate) matched_rule_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct BpmnCliEventFixture {
    #[serde(rename = "event_polls")]
    pub(crate) poll: BTreeMap<String, BpmnCliEventPollFixture>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BpmnCliEventPollFixture {
    pub(crate) ready: bool,
    #[serde(default)]
    pub(crate) winning_wait_id: Option<String>,
    #[serde(default = "empty_json_object")]
    pub(crate) data: serde_json::Value,
}
