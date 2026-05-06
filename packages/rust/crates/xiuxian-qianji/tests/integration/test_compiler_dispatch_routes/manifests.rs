pub(super) const KNOWLEDGE_MANIFEST: &str = r#"
name = "KnowledgeDispatch"

[[nodes]]
id = "Knowledge"
task_type = "knowledge"
weight = 1.0
params = {}
"#;

pub(super) const COMMAND_MANIFEST: &str = r#"
name = "CommandDispatch"

[[nodes]]
id = "Command"
task_type = "command"
weight = 1.0
params = { cmd = "echo hi", output_key = "stdout" }
"#;

pub(super) const HTTP_CALL_MANIFEST: &str = r#"
name = "HttpCallDispatch"

[[nodes]]
id = "OpenNavigation"
kind = "http_call"
contract = "wendao.docs.navigation"
method = "GET"
path = "/api/docs/navigation"
query = { repo = "$repo", page_id = "$page_id", related_limit = 5, family_limit = 3 }
"#;

pub(super) const CLI_CALL_MANIFEST: &str = r#"
name = "CliCallDispatch"

[[nodes]]
id = "OpenNavigationCli"
kind = "cli_call"
contract = "wendao.docs.navigation"
argv = ["wendao", "docs", "navigation", "--repo", "$repo", "--page-id", "$page_id", "--related-limit", "5", "--family-limit", "3"]
"#;

pub(super) const HTTP_CALL_INVALID_PATH_MANIFEST: &str = r#"
name = "HttpCallInvalidPathDispatch"

[[nodes]]
id = "OpenNavigation"
kind = "http_call"
contract = "wendao.docs.navigation"
method = "GET"
path = "/api/docs/not-navigation"
query = { repo = "$repo", page_id = "$page_id", related_limit = 5, family_limit = 3 }
"#;

pub(super) const CLI_CALL_UNKNOWN_FLAG_MANIFEST: &str = r#"
name = "CliCallInvalidFlagDispatch"

[[nodes]]
id = "OpenNavigationCli"
kind = "cli_call"
contract = "wendao.docs.navigation"
argv = ["wendao", "docs", "navigation", "--repo", "$repo", "--page-id", "$page_id", "--nope", "3"]
"#;

pub(super) const WRITE_FILE_MANIFEST: &str = r#"
name = "WriteFileDispatch"

[[nodes]]
id = "WriteFile"
task_type = "write_file"
weight = 1.0
params = { path = "notes/out.txt", content = "hello", output_key = "write_file_result" }
"#;

pub(super) const SUSPEND_MANIFEST: &str = r#"
name = "SuspendDispatch"

[[nodes]]
id = "Suspend"
task_type = "suspend"
weight = 1.0
params = { reason = "manual-check", prompt = "continue?", resume_key = "resume" }
"#;

pub(super) const ROUTER_MANIFEST: &str = r#"
name = "RouterDispatch"

[[nodes]]
id = "Router"
task_type = "router"
weight = 1.0
params = { branches = [["A", 0.6], ["B", 0.4]] }
"#;

pub(super) const ROUTER_SEMANTIC_GUARD_MANIFEST: &str = r#"
name = "RouterSemanticGuardDispatch"

[[nodes]]
id = "Router"
task_type = "router"
weight = 1.0
params = { branches = [["continue", 0.6], ["review_required", 0.4]], semantic_guard_route = true }
"#;

pub(super) const ROUTER_INVALID_WEIGHT_MANIFEST: &str = r#"
name = "RouterInvalidWeightDispatch"

[[nodes]]
id = "Router"
task_type = "router"
weight = 1.0
params = { branches = [["A", "not-a-number"]] }
"#;

pub(super) const CALIBRATION_MANIFEST: &str = r#"
name = "CalibrationDispatch"

[[nodes]]
id = "Calibration"
task_type = "calibration"
weight = 1.0
params = { target_node_id = "TargetNode" }
"#;

pub(super) const MOCK_MANIFEST: &str = r#"
name = "MockDispatch"

[[nodes]]
id = "MockNode"
task_type = "mock"
weight = 1.0
params = {}
"#;

pub(super) const SECURITY_SCAN_MANIFEST: &str = r#"
name = "SecurityScanDispatch"

[[nodes]]
id = "SecurityScan"
task_type = "security_scan"
weight = 1.0
params = { files_key = "changed_files", output_key = "issues", abort_on_violation = true }
"#;

pub(super) const ANNOTATION_EXPLICIT_AFFINITY_MANIFEST: &str = r#"
name = "AnnotationExplicitAffinityDispatch"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = { agent_id = "agent-alpha", role_class = "planner" }
"#;

pub(super) const ANNOTATION_DERIVED_AFFINITY_MANIFEST: &str = r#"
name = "AnnotationDerivedAffinityDispatch"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "semantic://personas/Steward.md"
"#;

pub(super) const FORMAL_AUDIT_NATIVE_MANIFEST: &str = r#"
name = "FormalAuditNativeDispatch"

[[nodes]]
id = "Teacher"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Steward"] }
"#;

pub(super) const FORMAL_AUDIT_NATIVE_WITH_MAX_RETRIES_MANIFEST: &str = r#"
name = "FormalAuditNativeWithMaxRetriesDispatch"

[[nodes]]
id = "Teacher"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Steward"], max_retries = 2 }
"#;

#[cfg(not(feature = "llm"))]
pub(super) const LLM_TASK_MANIFEST: &str = r#"
name = "LlmDispatch"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { prompt = "Analyze", output_key = "analysis" }
"#;

pub(super) const WENDAO_INGESTER_MANIFEST: &str = r#"
name = "WendaoIngesterDispatch"

[[nodes]]
id = "WendaoIngester"
task_type = "wendao_ingester"
weight = 1.0
params = {}
"#;

pub(super) const WENDAO_REFRESH_MANIFEST: &str = r#"
name = "WendaoRefreshDispatch"

[[nodes]]
id = "WendaoRefresh"
task_type = "wendao_refresh"
weight = 1.0
params = {}
"#;

pub(super) const WENDAO_SQL_DISCOVER_MANIFEST: &str = r#"
name = "WendaoSqlDiscoverDispatch"

[[nodes]]
id = "WendaoSqlDiscover"
task_type = "wendao_sql_discover"
weight = 1.0
params = { endpoint = "http://127.0.0.1:39001/query" }
"#;

pub(super) const WENDAO_SQL_VALIDATE_MANIFEST: &str = r#"
name = "WendaoSqlValidateDispatch"

[[nodes]]
id = "WendaoSqlValidate"
task_type = "wendao_sql_validate"
weight = 1.0
params = {}
"#;

pub(super) const WENDAO_SQL_EXECUTE_MANIFEST: &str = r#"
name = "WendaoSqlExecuteDispatch"

[[nodes]]
id = "WendaoSqlExecute"
task_type = "wendao_sql_execute"
weight = 1.0
params = { endpoint = "http://127.0.0.1:39001/query" }
"#;

#[cfg(not(feature = "llm"))]
pub(super) const FORMAL_AUDIT_LLM_MANIFEST: &str = r#"
name = "FormalAuditDispatch"

[[nodes]]
id = "Teacher"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Steward"] }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
provider = "openai"
model = "gpt-4o-mini"
"#;

pub(super) const UNKNOWN_TASK_MANIFEST: &str = r#"
name = "UnknownDispatch"

[[nodes]]
id = "Unknown"
task_type = "not_real_task"
weight = 1.0
params = {}
"#;
