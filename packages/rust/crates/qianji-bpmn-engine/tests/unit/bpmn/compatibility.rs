use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnHostBridge, BpmnInstanceInit, BpmnParseOptions,
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, LintIssue, ManualTaskOutcome, ManualTaskRequest, PendingHostWorkRequest,
    PendingHostWorkResult, ScriptTaskOutcome, ScriptTaskRequest, SendTaskOutcome, SendTaskRequest,
    ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome, UserTaskRequest, advance_instance,
    build_pending_host_work_request, create_instance, lint_bpmn_source, parse_bpmn_package,
    snapshot_bpmn_source,
};
use serde_json::json;
use std::sync::Arc;

use super::fixture_source;

const COMPATIBILITY_FIXTURE: &str = "native-bpmn-js-compatibility.bpmn";

#[test]
fn native_compatibility_fixture_uses_only_standard_xml_surface() {
    let source = compatibility_source();

    assert_contains(
        &source.contents,
        "http://www.omg.org/spec/BPMN/20100524/MODEL",
    );
    assert_contains(&source.contents, "http://www.omg.org/spec/BPMN/20100524/DI");
    assert_contains(&source.contents, "http://www.omg.org/spec/DD/20100524/DC");
    assert_contains(&source.contents, "http://www.omg.org/spec/DD/20100524/DI");
    assert_not_contains(&source.contents, "qianji:");
    assert_not_contains(&source.contents, "xmlns:qianji");
}

#[test]
fn native_compatibility_fixture_snapshots_standard_di_and_task_io() {
    let source = compatibility_source();
    let snapshot = snapshot_bpmn_source(&source)
        .must("native compatibility BPMN should produce a document snapshot");

    assert_eq!(snapshot.root.element_name, "definitions");
    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("http://www.omg.org/spec/BPMN/20100524/MODEL")
    );
    assert_eq!(
        snapshot.root.target_namespace.as_deref(),
        Some("https://example.com/bpmn/native-compatibility")
    );
    assert_eq!(snapshot.root.process_count, 1);
    assert_eq!(snapshot.root.diagram_count, 1);

    let diagram = &snapshot.root.diagrams[0];
    let plane = diagram
        .plane
        .as_ref()
        .must("native compatibility fixture should preserve a BPMNPlane");
    assert_eq!(plane.shapes.len(), 3);
    assert_eq!(plane.edges.len(), 2);

    let package = parse_bpmn_package(&[source], &BpmnParseOptions::default())
        .must("native compatibility BPMN should parse without custom XML");
    let process = package
        .find_process("native_compatibility")
        .must("native compatibility process should be present");
    assert_eq!(process.data_object_bindings.len(), 2);

    let task = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "ServiceTask_ReviewOrder")
        .must("native compatibility fixture should preserve the service task");
    let task_io = task
        .task_io
        .as_ref()
        .must("service task should preserve native task IO");
    assert_eq!(task_io.inputs.len(), 2);
    assert_eq!(task_io.outputs.len(), 1);
}

#[test]
fn native_compatibility_lint_reports_only_standard_di_metadata() {
    let report = lint_bpmn_source(&compatibility_source());

    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = single_issue(&report.issues, "bpmn.metadata_di_surface");
    assert_eq!(issue.evidence["snapshot"]["diagram_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["shape_count"], 3);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["edge_count"], 2);
}

#[tokio::test(flavor = "current_thread")]
async fn native_compatibility_fixture_runs_with_native_task_io() {
    let package = Arc::new(
        parse_bpmn_package(&[compatibility_source()], &BpmnParseOptions::default())
            .must("native compatibility BPMN should parse for runtime"),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "native_compatibility",
        BpmnInstanceInit::new(
            "wf_native_compatibility",
            json!({ "OrderData": { "id": "A-100", "amount": 42 } }),
            10,
        ),
    )
    .must("native compatibility instance should be created");
    let host = CompatibilityHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("native compatibility process should block on service task");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let request = build_pending_host_work_request(&instance)
        .must("blocked native compatibility instance should expose host work");
    let PendingHostWorkRequest::Service(request) = request else {
        panic!("native compatibility fixture should block on service work");
    };

    assert_eq!(
        request.inputs,
        json!({
            "order": { "id": "A-100", "amount": 42 },
            "policy": { "mode": "compatibility", "channel": "native-bpmn" }
        })
    );
    assert_eq!(request.output_bindings.len(), 1);
    assert_eq!(request.output_bindings[0].name.as_ref(), "decision");
    assert_eq!(request.output_bindings[0].target_ref.as_ref(), "OrderData");

    crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        request.token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "decision": { "approved": true, "reviewed_by": "host" } }),
        }),
        100,
    )
    .must("native compatibility output should map through dataOutputAssociation");

    assert_eq!(
        instance.variables["OrderData"],
        json!({ "approved": true, "reviewed_by": "host" })
    );

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("native compatibility process should complete after mapped output");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}

fn compatibility_source() -> qianji_bpmn_engine::BpmnSourceFile {
    fixture_source(COMPATIBILITY_FIXTURE)
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected compatibility fixture to contain {needle}"
    );
}

fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase()),
        "compatibility fixture must not contain {needle}"
    );
}

fn single_issue<'a>(issues: &'a [LintIssue], code: &str) -> &'a LintIssue {
    issues
        .iter()
        .find(|issue| issue.code == code)
        .unwrap_or_else(|| panic!("expected lint issue {code}"))
}

struct CompatibilityHost {
    now_ms: u64,
}

impl CompatibilityHost {
    fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for CompatibilityHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        panic!("compatibility tests should not dispatch host work");
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        panic!("compatibility tests should not poll external events");
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}
