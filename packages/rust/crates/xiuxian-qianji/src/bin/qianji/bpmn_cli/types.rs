use super::deps::{
    BTreeMap, Deserialize, Path, PathBuf, QianjiBpmnCheckpointStore, QianjiBpmnHostBridge,
    empty_json_object,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BpmnCliCommand {
    Run(BpmnRunCliCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnRunCliCommand {
    pub(crate) bpmn_path: PathBuf,
    pub(crate) dmn_paths: Vec<PathBuf>,
    pub(crate) process_id: String,
    pub(crate) instance_id: String,
    pub(crate) context_json: Option<String>,
    pub(crate) checkpoint_backend: Option<BpmnCliCheckpointBackend>,
    pub(crate) host_fixture_path: Option<PathBuf>,
    pub(crate) event_fixture_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BpmnCliCheckpointBackend {
    RuntimeValkey,
    #[cfg(feature = "sqlite")]
    Sqlite(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BpmnCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}

pub(crate) struct BpmnRunRenderContext<'a> {
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
    #[serde(rename = "service_tasks")]
    pub(crate) service: BTreeMap<String, BpmnCliHostDataFixture>,
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
