use crate::bpmn::{
    QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, QIANJI_RUN_CONSOLE_EVENT_ROUTE,
    QIANJI_RUN_CONSOLE_RUN_ID_HEADER, QIANJI_RUN_CONSOLE_SCHEMA_VERSION,
    QianjiRunConsoleFlightService,
};
use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightData, FlightDescriptor, FlightInfo, Ticket};
use futures::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tonic::metadata::{Ascii, MetadataMap, MetadataValue};
use tonic::{Request, Status};
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, ControlResult, InMemoryControlLedger, RunId,
    StepId,
};
use xiuxian_security::{
    InternalServiceSecurity, PublicProtocolSurface, SignedPrincipalSigner,
    WENDAO_AUTH_SCOPE_HEADER, WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
    WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER, WENDAO_PUBLIC_PROTOCOL_HEADER,
    WENDAO_SIGNED_PRINCIPAL_HEADER,
};

#[tokio::test]
async fn qianji_run_console_flight_streams_event_rows() {
    let run_id = run_id();
    let service = service_with_run(&run_id);

    let flight_info = fetch_flight_info(&service, QIANJI_RUN_CONSOLE_EVENT_ROUTE, &run_id).await;
    assert_eq!(ticket_string(&flight_info), QIANJI_RUN_CONSOLE_EVENT_ROUTE);
    assert_eq!(flight_info.total_records, 3);

    let batches = collect_route_batches(&service, QIANJI_RUN_CONSOLE_EVENT_ROUTE, &run_id).await;
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].num_rows(), 3);
    assert_eq!(string_value(&batches[0], "runId", 0), run_id.as_str());
    assert_eq!(string_value(&batches[0], "kind", 2), "step_succeeded");
}

#[tokio::test]
async fn qianji_run_console_flight_streams_element_state_rows() {
    let run_id = run_id();
    let service = service_with_run(&run_id);

    let flight_info =
        fetch_flight_info(&service, QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, &run_id).await;
    assert_eq!(
        ticket_string(&flight_info),
        QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE
    );
    assert_eq!(flight_info.total_records, 1);

    let batches =
        collect_route_batches(&service, QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, &run_id).await;
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].num_rows(), 1);
    assert_eq!(string_value(&batches[0], "elementId", 0), "review");
    assert_eq!(string_value(&batches[0], "state", 0), "completed");
}

#[tokio::test]
async fn qianji_run_console_flight_rejects_missing_run_id() {
    let run_id = run_id();
    let service = service_with_run(&run_id);
    let mut request = Request::new(FlightDescriptor::new_path(vec![
        QIANJI_RUN_CONSOLE_EVENT_ROUTE.to_string(),
    ]));
    insert_schema_version(request.metadata_mut());

    let Err(error) = service.get_flight_info(request).await else {
        panic!("missing run id should fail");
    };
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn qianji_run_console_flight_rejects_wrong_schema_version() {
    let run_id = run_id();
    let service = service_with_run(&run_id);
    let mut request = Request::new(FlightDescriptor::new_path(vec![
        QIANJI_RUN_CONSOLE_EVENT_ROUTE.to_string(),
    ]));
    insert_run_id(request.metadata_mut(), &run_id);
    request
        .metadata_mut()
        .insert("x-wendao-schema-version", metadata_value("wrong.version"));

    let Err(error) = service.get_flight_info(request).await else {
        panic!("wrong schema version should fail");
    };
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn qianji_run_console_flight_rejects_missing_internal_principal_when_secured() {
    let run_id = run_id();
    let service = secured_service_with_run(&run_id);
    let mut request = Request::new(FlightDescriptor::new_path(vec![
        QIANJI_RUN_CONSOLE_EVENT_ROUTE.to_string(),
    ]));
    insert_run_id(request.metadata_mut(), &run_id);
    insert_schema_version(request.metadata_mut());

    let Err(error) = service.get_flight_info(request).await else {
        panic!("missing internal principal should fail");
    };
    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    assert!(
        error
            .message()
            .contains("missing internal service identity")
    );
}

#[tokio::test]
async fn qianji_run_console_flight_accepts_gateway_signed_principal_when_secured() {
    let run_id = run_id();
    let service = secured_service_with_run(&run_id);
    let mut request = Request::new(FlightDescriptor::new_path(vec![
        QIANJI_RUN_CONSOLE_EVENT_ROUTE.to_string(),
    ]));
    insert_run_id(request.metadata_mut(), &run_id);
    insert_schema_version(request.metadata_mut());
    insert_internal_principal(request.metadata_mut(), "internal-secret");

    let flight_info = service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("signed internal principal should pass: {error}"))
        .into_inner();

    assert_eq!(ticket_string(&flight_info), QIANJI_RUN_CONSOLE_EVENT_ROUTE);
    assert_eq!(flight_info.total_records, 3);
}

fn service_with_run(run_id: &RunId) -> QianjiRunConsoleFlightService {
    let ledger = InMemoryControlLedger::new();
    append_event(
        &ledger,
        ControlEvent::run(
            run_id.clone(),
            10,
            ControlEventKind::RunCreated {
                intent: "operator start".to_owned(),
                budget: None,
                metadata: serde_json::Value::Null,
            },
        ),
    );
    append_event(
        &ledger,
        ControlEvent::step(
            run_id.clone(),
            step_id("review"),
            11,
            ControlEventKind::StepStarted,
        ),
    );
    append_event(
        &ledger,
        ControlEvent::step(
            run_id.clone(),
            step_id("review"),
            12,
            ControlEventKind::StepSucceeded,
        ),
    );
    QianjiRunConsoleFlightService::new(Arc::new(ledger))
}

fn secured_service_with_run(run_id: &RunId) -> QianjiRunConsoleFlightService {
    service_with_run(run_id).with_internal_security(InternalServiceSecurity::gateway(
        Arc::<str>::from("internal-secret"),
        Arc::<str>::from("QIANJI_INTERNAL_PRINCIPAL_REQUIRED"),
    ))
}

async fn fetch_flight_info(
    service: &QianjiRunConsoleFlightService,
    route: &str,
    run_id: &RunId,
) -> FlightInfo {
    let mut request = Request::new(FlightDescriptor::new_path(vec![route.to_string()]));
    insert_run_id(request.metadata_mut(), run_id);
    insert_schema_version(request.metadata_mut());
    service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("{route} FlightInfo should load: {error}"))
        .into_inner()
}

async fn collect_route_batches(
    service: &QianjiRunConsoleFlightService,
    route: &str,
    run_id: &RunId,
) -> Vec<RecordBatch> {
    let flight_info = fetch_flight_info(service, route, run_id).await;
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .unwrap_or_else(|| panic!("{route} should provide a ticket"));
    let mut request = Request::new(ticket);
    insert_run_id(request.metadata_mut(), run_id);
    insert_schema_version(request.metadata_mut());
    let frames = service
        .do_get(request)
        .await
        .unwrap_or_else(|error| panic!("{route} do_get should stream: {error}"))
        .into_inner()
        .collect::<Vec<_>>()
        .await;
    decode_flight_batches(frames, route).await
}

async fn decode_flight_batches(
    frames: Vec<Result<FlightData, Status>>,
    context: &str,
) -> Vec<RecordBatch> {
    let stream = futures::stream::iter(
        frames
            .into_iter()
            .map(|frame| frame.map_err(arrow_flight::error::FlightError::from)),
    );
    let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(stream);
    let mut batches = Vec::new();
    while let Some(batch) = batch_stream
        .try_next()
        .await
        .unwrap_or_else(|error| panic!("{context} should decode Flight batches: {error}"))
    {
        batches.push(batch);
    }
    batches
}

fn ticket_string(flight_info: &FlightInfo) -> String {
    let ticket: &Ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.as_ref())
        .unwrap_or_else(|| panic!("FlightInfo should include a ticket"));
    String::from_utf8_lossy(ticket.ticket.as_ref()).into_owned()
}

fn insert_run_id(metadata: &mut MetadataMap, run_id: &RunId) {
    metadata.insert(
        QIANJI_RUN_CONSOLE_RUN_ID_HEADER,
        metadata_value(run_id.as_str()),
    );
}

fn insert_schema_version(metadata: &mut MetadataMap) {
    metadata.insert(
        "x-wendao-schema-version",
        metadata_value(QIANJI_RUN_CONSOLE_SCHEMA_VERSION),
    );
}

fn insert_internal_principal(metadata: &mut MetadataMap, signing_secret: &str) {
    let surface = PublicProtocolSurface::ArrowFlight;
    let signed_principal = SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from(signing_secret),
    )
    .sign_user_token(surface, "public-token");
    metadata.insert(
        WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
        metadata_value(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
    );
    metadata.insert(
        WENDAO_PUBLIC_PROTOCOL_HEADER,
        metadata_value(surface.protocol()),
    );
    metadata.insert(WENDAO_AUTH_SCOPE_HEADER, metadata_value(surface.scope()));
    metadata.insert(
        WENDAO_SIGNED_PRINCIPAL_HEADER,
        metadata_value(signed_principal.as_str()),
    );
}

fn metadata_value(raw: &str) -> MetadataValue<Ascii> {
    MetadataValue::try_from(raw).unwrap_or_else(|error| panic!("metadata should be valid: {error}"))
}

fn string_value<'a>(batch: &'a RecordBatch, column: &str, row: usize) -> &'a str {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("{column} should be a StringArray"))
        .value(row)
}

fn append_event(ledger: &InMemoryControlLedger, event: ControlEvent) {
    let result: ControlResult<_> = ledger.append_event(event);
    result.unwrap_or_else(|error| panic!("control event should append: {error}"));
}

fn run_id() -> RunId {
    RunId::new("bpmn.workflow.run-console-flight")
        .unwrap_or_else(|error| panic!("run id should be valid: {error}"))
}

fn step_id(value: &str) -> StepId {
    StepId::new(value).unwrap_or_else(|error| panic!("step id should be valid: {error}"))
}
