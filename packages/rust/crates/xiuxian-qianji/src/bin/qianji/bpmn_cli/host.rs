use super::deps::{
    Arc, BTreeMap, BpmnPackage, BusinessRuleTaskOutcome, BusinessRuleTaskRequest,
    DmnEvaluationResult, EventPollOutcome, EventPollRequest, HostBridgeError, ManualTaskOutcome,
    ManualTaskRequest, Path, QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder, Ready,
    SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, StdArc,
    UserTaskOutcome, UserTaskRequest, fs, io, ready, resolve_cli_path,
};
use super::types::{
    BpmnCliBusinessRuleFixture, BpmnCliEventFixture, BpmnCliEventPollFixture,
    BpmnCliHostBridgeContext, BpmnCliHostDataFixture, BpmnCliHostFixture, BpmnRunCliCommand,
};

pub(super) fn build_bpmn_cli_host_bridge(
    package: &Arc<BpmnPackage>,
    command: &BpmnRunCliCommand,
) -> Result<BpmnCliHostBridgeContext, Box<dyn std::error::Error>> {
    let resolved_host_fixture_path = command
        .host_fixture_path
        .as_deref()
        .map(resolve_cli_path)
        .transpose()?;
    let resolved_event_fixture_path = command
        .event_fixture_path
        .as_deref()
        .map(resolve_cli_path)
        .transpose()?;
    let host_fixture = resolved_host_fixture_path
        .as_deref()
        .map(load_bpmn_cli_host_fixture)
        .transpose()?;
    let event_fixture = resolved_event_fixture_path
        .as_deref()
        .map(load_bpmn_cli_event_fixture)
        .transpose()?;

    let host = match (&host_fixture, &event_fixture) {
        (None, None) => QianjiBpmnHostBridge::default(),
        _ => build_fixture_backed_bpmn_host_bridge(
            build_bpmn_cli_process_node_ids(package.as_ref(), &command.process_id)?,
            &command.process_id,
            host_fixture,
            event_fixture,
        ),
    };

    Ok(BpmnCliHostBridgeContext {
        host,
        resolved_host_fixture_path,
        resolved_event_fixture_path,
    })
}

fn load_bpmn_cli_host_fixture(
    path: &Path,
) -> Result<BpmnCliHostFixture, Box<dyn std::error::Error>> {
    let raw_fixture = fs::read_to_string(path).map_err(|error| {
        io::Error::other(format!(
            "failed to read `--host-fixture` file at {}: {error}",
            path.display()
        ))
    })?;
    serde_json::from_str(&raw_fixture).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to parse `--host-fixture` as valid JSON: {error}"),
        )
        .into()
    })
}

fn load_bpmn_cli_event_fixture(
    path: &Path,
) -> Result<BpmnCliEventFixture, Box<dyn std::error::Error>> {
    let raw_fixture = fs::read_to_string(path).map_err(|error| {
        io::Error::other(format!(
            "failed to read `--event-fixture` file at {}: {error}",
            path.display()
        ))
    })?;
    serde_json::from_str(&raw_fixture).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to parse `--event-fixture` as valid JSON: {error}"),
        )
        .into()
    })
}

fn build_bpmn_cli_process_node_ids(
    package: &BpmnPackage,
    process_id: &str,
) -> io::Result<StdArc<Vec<String>>> {
    let process = package.find_process(process_id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "BPMN package '{}' does not contain process '{process_id}'",
                package.package_id
            ),
        )
    })?;
    Ok(StdArc::new(
        process
            .nodes
            .iter()
            .map(|node| node.bpmn_id.to_string())
            .collect::<Vec<_>>(),
    ))
}

fn build_fixture_backed_bpmn_host_bridge(
    node_ids: StdArc<Vec<String>>,
    process_id: &str,
    host_fixture: Option<BpmnCliHostFixture>,
    event_fixture: Option<BpmnCliEventFixture>,
) -> QianjiBpmnHostBridge {
    let process_id = process_id.to_string();
    let mut builder = QianjiBpmnHostBridge::builder();

    if let Some(host_fixture) = host_fixture {
        builder = builder_for_bpmn_cli_host_fixture(
            builder,
            StdArc::clone(&node_ids),
            &process_id,
            host_fixture,
        );
    }

    if let Some(event_fixture) = event_fixture {
        builder = builder.on_event_poll(event_poll_fixture_handler(
            node_ids,
            process_id,
            StdArc::new(event_fixture.poll),
        ));
    }

    builder.build()
}

fn builder_for_bpmn_cli_host_fixture(
    builder: QianjiBpmnHostBridgeBuilder,
    node_ids: StdArc<Vec<String>>,
    process_id: &str,
    fixture: BpmnCliHostFixture,
) -> QianjiBpmnHostBridgeBuilder {
    let send = StdArc::new(fixture.send);
    let service = StdArc::new(fixture.service);
    let user = StdArc::new(fixture.user);
    let manual = StdArc::new(fixture.manual);
    let business_rule = StdArc::new(fixture.business_rule);

    builder
        .on_send_task(send_task_fixture_handler(
            StdArc::clone(&node_ids),
            process_id.to_string(),
            send,
        ))
        .on_service_task(service_task_fixture_handler(
            StdArc::clone(&node_ids),
            process_id.to_string(),
            service,
        ))
        .on_user_task(user_task_fixture_handler(
            StdArc::clone(&node_ids),
            process_id.to_string(),
            user,
        ))
        .on_manual_task(manual_task_fixture_handler(
            StdArc::clone(&node_ids),
            process_id.to_string(),
            manual,
        ))
        .on_business_rule_task(business_rule_task_fixture_handler(
            node_ids,
            process_id.to_string(),
            business_rule,
        ))
}

fn send_task_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliHostDataFixture>>,
) -> impl Fn(SendTaskRequest) -> Ready<Result<SendTaskOutcome, HostBridgeError>> + Send + Sync + 'static
{
    move |request| {
        ready(
            resolve_bpmn_cli_host_data_fixture(
                node_ids.as_slice(),
                &process_id,
                request.node_index,
                "send_tasks",
                fixture.as_ref(),
            )
            .map(|entry| SendTaskOutcome {
                data: entry.data.clone(),
            }),
        )
    }
}

fn service_task_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliHostDataFixture>>,
) -> impl Fn(ServiceTaskRequest) -> Ready<Result<ServiceTaskOutcome, HostBridgeError>>
+ Send
+ Sync
+ 'static {
    move |request| {
        ready(
            resolve_bpmn_cli_host_data_fixture(
                node_ids.as_slice(),
                &process_id,
                request.node_index,
                "service_tasks",
                fixture.as_ref(),
            )
            .map(|entry| ServiceTaskOutcome {
                data: entry.data.clone(),
            }),
        )
    }
}

fn user_task_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliHostDataFixture>>,
) -> impl Fn(UserTaskRequest) -> Ready<Result<UserTaskOutcome, HostBridgeError>> + Send + Sync + 'static
{
    move |request| {
        ready(
            resolve_bpmn_cli_host_data_fixture(
                node_ids.as_slice(),
                &process_id,
                request.node_index,
                "user_tasks",
                fixture.as_ref(),
            )
            .map(|entry| UserTaskOutcome {
                data: entry.data.clone(),
            }),
        )
    }
}

fn manual_task_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliHostDataFixture>>,
) -> impl Fn(ManualTaskRequest) -> Ready<Result<ManualTaskOutcome, HostBridgeError>>
+ Send
+ Sync
+ 'static {
    move |request| {
        ready(
            resolve_bpmn_cli_host_data_fixture(
                node_ids.as_slice(),
                &process_id,
                request.node_index,
                "manual_tasks",
                fixture.as_ref(),
            )
            .map(|entry| ManualTaskOutcome {
                data: entry.data.clone(),
            }),
        )
    }
}

fn business_rule_task_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliBusinessRuleFixture>>,
) -> impl Fn(BusinessRuleTaskRequest) -> Ready<Result<BusinessRuleTaskOutcome, HostBridgeError>>
+ Send
+ Sync
+ 'static {
    move |request| {
        ready(
            resolve_bpmn_cli_business_rule_fixture(
                node_ids.as_slice(),
                &process_id,
                request.node_index,
                fixture.as_ref(),
            )
            .map(|entry| BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new(
                    request.evaluation.decision.decision_id.as_ref(),
                    entry.output.clone(),
                    entry
                        .matched_rule_ids
                        .iter()
                        .map(|rule_id: &String| Arc::<str>::from(rule_id.as_str()))
                        .collect(),
                ),
            }),
        )
    }
}

fn resolve_bpmn_cli_host_data_fixture<'a>(
    node_ids: &'a [String],
    process_id: &str,
    node_index: u32,
    bucket: &str,
    fixture: &'a BTreeMap<String, BpmnCliHostDataFixture>,
) -> Result<&'a BpmnCliHostDataFixture, HostBridgeError> {
    let node_id = resolve_bpmn_host_fixture_node_id(node_ids, process_id, node_index)?;
    fixture
        .get(node_id)
        .ok_or_else(|| missing_bpmn_host_fixture_entry(process_id, bucket, node_id))
}

fn resolve_bpmn_cli_business_rule_fixture<'a>(
    node_ids: &'a [String],
    process_id: &str,
    node_index: u32,
    fixture: &'a BTreeMap<String, BpmnCliBusinessRuleFixture>,
) -> Result<&'a BpmnCliBusinessRuleFixture, HostBridgeError> {
    let node_id = resolve_bpmn_host_fixture_node_id(node_ids, process_id, node_index)?;
    fixture
        .get(node_id)
        .ok_or_else(|| missing_bpmn_host_fixture_entry(process_id, "business_rule_tasks", node_id))
}

fn event_poll_fixture_handler(
    node_ids: StdArc<Vec<String>>,
    process_id: String,
    fixture: StdArc<BTreeMap<String, BpmnCliEventPollFixture>>,
) -> impl Fn(EventPollRequest) -> Ready<Result<EventPollOutcome, HostBridgeError>> + Send + Sync + 'static
{
    move |request| {
        ready(
            resolve_bpmn_cli_event_poll_fixture(
                node_ids.as_slice(),
                &process_id,
                &request,
                fixture.as_ref(),
            )
            .and_then(|entry| {
                Ok(EventPollOutcome {
                    ready: entry.ready,
                    winning_wait_node_index: resolve_bpmn_cli_event_poll_winner(
                        node_ids.as_slice(),
                        &process_id,
                        &request,
                        entry,
                    )?,
                    data: entry.data.clone(),
                })
            }),
        )
    }
}

fn resolve_bpmn_cli_event_poll_fixture<'a>(
    node_ids: &'a [String],
    process_id: &str,
    request: &EventPollRequest,
    fixture: &'a BTreeMap<String, BpmnCliEventPollFixture>,
) -> Result<&'a BpmnCliEventPollFixture, HostBridgeError> {
    let poll_key = bpmn_cli_event_poll_key(node_ids, process_id, request)?;
    fixture.get(&poll_key).ok_or_else(|| {
        HostBridgeError::RequestFailed(format!(
            "event fixture missing `event_polls.{poll_key}` for process '{process_id}'; keys are active BPMN wait ids joined by `|` in sorted order"
        ))
    })
}

fn resolve_bpmn_cli_event_poll_winner(
    node_ids: &[String],
    process_id: &str,
    request: &EventPollRequest,
    entry: &BpmnCliEventPollFixture,
) -> Result<Option<u32>, HostBridgeError> {
    match (entry.ready, request.waits.len(), entry.winning_wait_id.as_deref()) {
        (true, 0, _) => Err(HostBridgeError::RequestFailed(format!(
            "event fixture cannot resolve a winning wait for process '{process_id}' because the poll request has no waits"
        ))),
        (false, _, _) | (true, 1, None) => Ok(None),
        (true, _, Some(wait_id)) => request
            .waits
            .iter()
            .find(|wait| {
                resolve_bpmn_host_fixture_node_id(node_ids, process_id, wait.node_index)
                    .ok()
                    .is_some_and(|node_id| node_id == wait_id)
            })
            .map(|wait| wait.node_index)
            .ok_or_else(|| {
                HostBridgeError::RequestFailed(format!(
                    "event fixture winning wait '{wait_id}' is not active for process '{process_id}'"
                ))
            })
            .map(Some),
        (true, _, None) => Err(HostBridgeError::RequestFailed(format!(
            "event fixture requires `winning_wait_id` when multiple waits compete in process '{process_id}'"
        ))),
    }
}

fn bpmn_cli_event_poll_key(
    node_ids: &[String],
    process_id: &str,
    request: &EventPollRequest,
) -> Result<String, HostBridgeError> {
    let mut wait_ids = request
        .waits
        .iter()
        .map(|wait| {
            resolve_bpmn_host_fixture_node_id(node_ids, process_id, wait.node_index)
                .map(ToString::to_string)
        })
        .collect::<Result<Vec<_>, _>>()?;
    wait_ids.sort();
    Ok(wait_ids.join("|"))
}

fn resolve_bpmn_host_fixture_node_id<'a>(
    node_ids: &'a [String],
    process_id: &str,
    node_index: u32,
) -> Result<&'a str, HostBridgeError> {
    node_ids
        .get(node_index as usize)
        .map(String::as_str)
        .ok_or_else(|| {
            HostBridgeError::RequestFailed(format!(
                "host fixture could not resolve BPMN node id for process '{process_id}' at node index {node_index}"
            ))
        })
}

fn missing_bpmn_host_fixture_entry(
    process_id: &str,
    bucket: &str,
    node_id: &str,
) -> HostBridgeError {
    HostBridgeError::RequestFailed(format!(
        "host fixture missing `{bucket}.{node_id}` for process '{process_id}'; fixture entries are keyed by BPMN node id"
    ))
}
