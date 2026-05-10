use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    time::Instant,
};

use super::{
    RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV,
    SearchStrategyFlowCandidateInputBatch, SearchStrategyFlowFlightMaterializationConfig,
    SearchStrategyFlowPersistentBatchHost,
    configured_wendaograph_search_strategy_flow_markdown_replay_families,
    configured_wendaograph_search_strategy_flow_markdown_replay_families_with_limit,
    enrich_wendaograph_search_strategy_flow_retrieval_routes,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_search_strategy_flow_probe_action, run_wendaograph_search_strategy_flow_json,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    search_strategy_flow_probe_action_route,
};
use arrow::{
    array::{Float64Array, Int32Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use arrow_flight::{
    Action, Criteria, Empty, FlightData, FlightDescriptor, FlightEndpoint, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, SchemaResult, Ticket,
    encode::FlightDataEncoderBuilder,
    flight_service_server::{FlightService, FlightServiceServer},
};
use async_trait::async_trait;
use futures::Stream;
use tokio::net::TcpListener;
use tokio_stream::{StreamExt, wrappers::TcpListenerStream};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_COLUMN, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
    REPO_SEARCH_NAVIGATION_PATH_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
    REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN,
};

type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
type PutResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::PutResult, Status>> + Send>>;
type ActionResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
type ActionTypeStream =
    Pin<Box<dyn Stream<Item = Result<arrow_flight::ActionType, Status>> + Send>>;

const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV: &str =
    "WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV: &str =
    "WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_TIMEOUT_SECONDS_ENV: &str =
    "WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_TIMEOUT_SECONDS";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT_ENV: &str =
    "WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_EXPECTED_SOURCE_FRAGMENTS_ENV: &str =
    "WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_EXPECTED_SOURCE_FRAGMENTS";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_INTENT: &str =
    "search strategy flow link graph python julia toml";
const WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_TIMEOUT_SECONDS: u64 = 30;

#[derive(Debug, Clone, Copy)]
struct SearchStrategyFlowConfiguredMarkdownReplaySpec {
    family: &'static str,
    intent: &'static str,
    include_dir: &'static str,
    expected_source_prefix: &'static str,
}

#[derive(Debug, Clone)]
struct SearchStrategyFlowConfiguredMarkdownReplayInput {
    spec: SearchStrategyFlowConfiguredMarkdownReplaySpec,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    batch: SearchStrategyFlowCandidateInputBatch,
}

#[derive(Debug, Clone, Copy)]
struct SearchStrategyFlowLiveReplayFamilyReport {
    family: &'static str,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    input_candidate_count: usize,
    trace_candidate_count: usize,
    frontier_count: usize,
    selected_frontier_count: usize,
    planner_action_count: usize,
    non_stop_planner_action_count: usize,
    route_count: usize,
    projected_row_count: usize,
}

#[derive(Debug, Clone)]
struct SearchStrategyFlowLiveReplayRunReport {
    family_reports: Vec<SearchStrategyFlowLiveReplayFamilyReport>,
    elapsed_ms: u128,
}

#[derive(Clone)]
struct SearchStrategyFlowFakeFlightService {
    batches_by_route: Arc<HashMap<String, RecordBatch>>,
}

#[path = "search_strategy/batch_profile.rs"]
mod batch_profile;
#[path = "search_strategy/materialized_bridge.rs"]
mod materialized_bridge;

#[derive(Clone, Copy)]
struct SearchStrategyFlowFakeFlightScenario {
    source_path: &'static str,
    doc_id: &'static str,
    title: &'static str,
    best_section: &'static str,
    line_start: i32,
    line_end: i32,
    page_id: &'static str,
    node_id: &'static str,
    node_anchor: &'static str,
    node_title: &'static str,
}

impl SearchStrategyFlowFakeFlightScenario {
    const fn markdown() -> Self {
        Self {
            source_path: "docs/30_search_strategy/30.01_search_strategy_flow.md",
            doc_id: "search-strategy-flow-doc",
            title: "SearchStrategyFlow",
            best_section: "Stage 1 Query Understanding",
            line_start: 10,
            line_end: 25,
            page_id: "repo:docs:projection:explanation:doc:repo:docs:doc:docs/30_search_strategy/30.01_search_strategy_flow.md",
            node_id: "node:stage-1-query-understanding",
            node_anchor: "stage-1-query-understanding",
            node_title: "Stage 1 Query Understanding",
        }
    }

    const fn rust_reference() -> Self {
        Self {
            source_path: "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
            doc_id: "repo:docs:doc:packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
            title: "Rust PPR Search Strategy",
            best_section: "PPR Runtime Search Strategy",
            line_start: 18,
            line_end: 54,
            page_id: "repo:docs:projection:reference:doc:repo:docs:doc:packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
            node_id: "node:ppr-runtime-search-strategy",
            node_anchor: "ppr-runtime-search-strategy",
            node_title: "PPR Runtime Search Strategy",
        }
    }
}

#[async_trait]
impl FlightService for SearchStrategyFlowFakeFlightService {
    type HandshakeStream = HandshakeStream;
    type ListFlightsStream = FlightInfoStream;
    type DoGetStream = FlightDataStream;
    type DoPutStream = PutResultStream;
    type DoExchangeStream = FlightDataStream;
    type DoActionStream = ActionResultStream;
    type ListActionsStream = ActionTypeStream;

    async fn handshake(
        &self,
        _request: Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("handshake is not used"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("list_flights is not used"))
    }

    async fn get_flight_info(
        &self,
        request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let route = descriptor_route(request.get_ref())?;
        if !self.batches_by_route.contains_key(route.as_str()) {
            return Err(Status::not_found(format!("unknown route `{route}`")));
        }
        Ok(Response::new(FlightInfo {
            schema: Vec::<u8>::new().into(),
            flight_descriptor: None,
            endpoint: vec![FlightEndpoint {
                ticket: Some(Ticket {
                    ticket: route.into_bytes().into(),
                }),
                location: Vec::new(),
                expiration_time: None,
                app_metadata: Vec::<u8>::new().into(),
            }],
            total_records: -1,
            total_bytes: -1,
            ordered: true,
            app_metadata: Vec::<u8>::new().into(),
        }))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented("poll_flight_info is not used"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("get_schema is not used"))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        let route = String::from_utf8(request.into_inner().ticket.to_vec())
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        let batch = self
            .batches_by_route
            .get(route.as_str())
            .ok_or_else(|| Status::not_found(format!("unknown ticket route `{route}`")))?
            .clone();
        let response_stream = FlightDataEncoderBuilder::new()
            .build(tokio_stream::iter(vec![Ok::<
                RecordBatch,
                arrow_flight::error::FlightError,
            >(batch)]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put is not used"))
    }

    async fn do_exchange(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented("do_exchange is not used"))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("do_action is not used"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("list_actions is not used"))
    }
}

fn descriptor_route(descriptor: &FlightDescriptor) -> Result<String, Status> {
    if descriptor.path.is_empty() {
        return Err(Status::invalid_argument("empty descriptor path"));
    }
    Ok(format!("/{}", descriptor.path.join("/")))
}

async fn spawn_fake_search_strategy_flow_flight_service() -> (String, tokio::task::JoinHandle<()>) {
    spawn_fake_search_strategy_flow_flight_service_for(
        SearchStrategyFlowFakeFlightScenario::markdown(),
    )
    .await
}

async fn spawn_fake_search_strategy_flow_flight_service_for(
    scenario: SearchStrategyFlowFakeFlightScenario,
) -> (String, tokio::task::JoinHandle<()>) {
    spawn_fake_search_strategy_flow_flight_service_with_batches(HashMap::from([
        (REPO_SEARCH_ROUTE.to_owned(), repo_search_batch(scenario)),
        (
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE.to_owned(),
            page_index_batch(scenario),
        ),
        (
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE.to_owned(),
            retrieval_context_batch(scenario),
        ),
        (GRAPH_NEIGHBORS_ROUTE.to_owned(), graph_neighbors_batch()),
    ]))
    .await
}

async fn spawn_fake_search_strategy_flow_candidate_discovery_service()
-> (String, tokio::task::JoinHandle<()>) {
    spawn_fake_search_strategy_flow_flight_service_with_batches(HashMap::from([(
        REPO_SEARCH_ROUTE.to_owned(),
        repo_search_candidate_discovery_batch(),
    )]))
    .await
}

async fn spawn_fake_search_strategy_flow_flight_service_with_batches(
    batches_by_route: HashMap<String, RecordBatch>,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("bind fake SearchStrategyFlow Flight service: {error}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("read fake SearchStrategyFlow listener address: {error}"));
    let service = SearchStrategyFlowFakeFlightService {
        batches_by_route: Arc::new(batches_by_route),
    };
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(FlightServiceServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap_or_else(|error| panic!("fake SearchStrategyFlow Flight service: {error}"));
    });
    (format!("http://{address}"), server)
}

fn repo_search_batch(scenario: SearchStrategyFlowFakeFlightScenario) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_DOC_ID_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_TITLE_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_BEST_SECTION_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_LINE_COLUMN, DataType::Int32, false),
            Field::new(
                REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
                DataType::Int32,
                false,
            ),
            Field::new(REPO_SEARCH_SCORE_COLUMN, DataType::Float64, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![scenario.source_path])),
            Arc::new(StringArray::from(vec![scenario.source_path])),
            Arc::new(StringArray::from(vec![scenario.doc_id])),
            Arc::new(StringArray::from(vec![scenario.title])),
            Arc::new(StringArray::from(vec![scenario.best_section])),
            Arc::new(Int32Array::from(vec![scenario.line_start])),
            Arc::new(Int32Array::from(vec![scenario.line_end])),
            Arc::new(Float64Array::from(vec![0.99])),
        ],
    )
    .unwrap_or_else(|error| panic!("repo search batch should build: {error}"))
}

fn repo_search_candidate_discovery_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_DOC_ID_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_TITLE_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_BEST_SECTION_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_LINE_COLUMN, DataType::Int32, false),
            Field::new(
                REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
                DataType::Int32,
                false,
            ),
            Field::new(REPO_SEARCH_SCORE_COLUMN, DataType::Float64, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![
                "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
                "wendao.toml",
                "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
                ".data/WendaoGraph.jl/src/reasoning/search_strategy_flow/frontier.jl",
            ])),
            Arc::new(StringArray::from(vec![
                "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
                "wendao.toml",
                "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
                ".data/WendaoGraph.jl/src/reasoning/search_strategy_flow/frontier.jl",
            ])),
            Arc::new(StringArray::from(vec![
                "repo:docs:doc:packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
                "repo:docs:doc:wendao.toml",
                "repo:docs:doc:packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
                "repo:docs:doc:.data/WendaoGraph.jl/src/reasoning/search_strategy_flow/frontier.jl",
            ])),
            Arc::new(StringArray::from(vec![
                "Rust PPR Search Strategy",
                "Wendao Repository Configuration",
                "Python Analyzer Worker",
                "Julia Frontier Strategy",
            ])),
            Arc::new(StringArray::from(vec![
                "PPR Runtime Search Strategy",
                "Wendao Repository Configuration",
                "Python Analyzer Worker",
                "Julia Frontier Strategy",
            ])),
            Arc::new(Int32Array::from(vec![18, 1, 42, 7])),
            Arc::new(Int32Array::from(vec![54, 36, 88, 34])),
            Arc::new(Float64Array::from(vec![0.99, 0.95, 0.91, 0.93])),
        ],
    )
    .unwrap_or_else(|error| panic!("repo-search candidate discovery batch should build: {error}"))
}

fn page_index_batch(scenario: SearchStrategyFlowFakeFlightScenario) -> RecordBatch {
    let roots_json = format!(
        r#"[{{"node_id":"{}","anchor":"{}","title":"{}","children":[]}}]"#,
        scenario.node_id, scenario.node_anchor, scenario.node_title
    );
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("pageId", DataType::Utf8, false),
            Field::new("rootCount", DataType::Int32, false),
            Field::new("rootsJson", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![scenario.page_id])),
            Arc::new(Int32Array::from(vec![1])),
            Arc::new(StringArray::from(vec![roots_json])),
        ],
    )
    .unwrap_or_else(|error| panic!("page-index batch should build: {error}"))
}

fn retrieval_context_batch(scenario: SearchStrategyFlowFakeFlightScenario) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("pageId", DataType::Utf8, false),
            Field::new("nodeId", DataType::Utf8, false),
            Field::new("centerJson", DataType::Utf8, false),
            Field::new("nodeContextJson", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![scenario.page_id])),
            Arc::new(StringArray::from(vec![scenario.node_id])),
            Arc::new(StringArray::from(vec![format!(
                r#"{{"title":"{}"}}"#,
                scenario.node_title
            )])),
            Arc::new(StringArray::from(vec![r#"{"neighbors":[]}"#])),
        ],
    )
    .unwrap_or_else(|error| panic!("retrieval-context batch should build: {error}"))
}

fn graph_neighbors_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "rowType",
            DataType::Utf8,
            false,
        )])),
        vec![Arc::new(StringArray::from(vec!["neighbor"]))],
    )
    .unwrap_or_else(|error| panic!("graph-neighbors batch should build: {error}"))
}

fn search_strategy_flow_configured_markdown_replay_specs()
-> [SearchStrategyFlowConfiguredMarkdownReplaySpec; 4] {
    [
        SearchStrategyFlowConfiguredMarkdownReplaySpec {
            family: "local-doc-authority-boundary",
            intent: "SearchStrategyFlow WorkingKnowledge memory layer promotion boundary",
            include_dir: "docs",
            expected_source_prefix: "docs/",
        },
        SearchStrategyFlowConfiguredMarkdownReplaySpec {
            family: "semantic-doc-working-knowledge",
            intent: "semantic graph execution graph authority invariant",
            include_dir: "semantic",
            expected_source_prefix: "semantic/",
        },
        SearchStrategyFlowConfiguredMarkdownReplaySpec {
            family: "rust-package-doc-implementation-boundary",
            intent: "Wendao package documentation code intelligence AST evidence downlink",
            include_dir: "packages/rust/crates/xiuxian-wendao",
            expected_source_prefix: "packages/rust/crates/xiuxian-wendao/",
        },
        SearchStrategyFlowConfiguredMarkdownReplaySpec {
            family: "benchmark-doc-adapter-boundary",
            intent: "Wendao knowledge retrieval benchmark adapter documentation scenario",
            include_dir: "packages/python/wendao-knowledge-retrieval-benchmark",
            expected_source_prefix: "packages/python/wendao-knowledge-retrieval-benchmark/",
        },
    ]
}

#[test]
fn search_strategy_flow_rust_bridge_rejects_blank_intent_before_launch() {
    let error = match run_wendaograph_search_strategy_flow_json("   ", ".") {
        Ok(trace) => panic!("blank intent should fail before launching Julia, got {trace}"),
        Err(error) => error,
    };

    assert_eq!(error, "SearchStrategyFlow intent must not be blank");
}

#[test]
fn wendaograph_search_strategy_flow_live_replay_runs_local_markdown_families_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow live replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report = run_configured_markdown_live_replay_reports(search_root.as_path(), None);
    assert_configured_markdown_live_replay_reports(&report.family_reports);
}

#[tokio::test]
async fn wendaograph_search_strategy_flow_live_flight_index_replay_runs_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow live Flight replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV}=1, {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV}, and {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV}"
        );
        return;
    }

    let base_url = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV);
    let repo_id = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV);
    let timeout_seconds = live_flight_timeout_seconds();
    let intent = optional_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT_ENV)
        .unwrap_or_else(|| WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_INTENT.to_owned());
    let expected_source_fragments = live_flight_expected_source_fragments();
    let search_root = search_strategy_flow_live_replay_search_root();
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, repo_id)
        .unwrap_or_else(|error| panic!("create live Flight materialization config: {error}"))
        .with_timeout_seconds(timeout_seconds);

    let trace = run_wendaograph_search_strategy_flow_json_with_flight_materialization(
        &intent,
        search_root.as_path(),
        Some(config),
    )
    .await
    .unwrap_or_else(|error| panic!("run live SearchStrategyFlow Flight index replay: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace)
        .unwrap_or_else(|error| panic!("parse live Flight replay trace: {error}"));

    assert_eq!(
        trace.get("candidateInputSource"),
        Some(&serde_json::json!("rust-flight-repo-search")),
        "live Flight replay must use Flight repo-search candidates"
    );
    assert!(
        trace
            .get("candidateInputCount")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "live Flight replay must discover at least one candidate"
    );
    let candidates = json_array(&trace, "candidates", "live-flight-index");
    let routes = json_array(&trace, "retrievalRoutes", "live-flight-index");
    let projected_rows = json_array(&trace, "rustProjectedEvidenceRows", "live-flight-index");

    assert!(!candidates.is_empty(), "live Flight candidates must exist");
    assert!(!routes.is_empty(), "live Flight routes must exist");
    assert!(
        !projected_rows.is_empty(),
        "live Flight projected evidence rows must exist"
    );
    assert_summary_matches_candidate_rows("live-flight-index", &trace, candidates);
    assert_strategy_flow_validation_flags("live-flight-index", &trace, true);
    assert_live_flight_routes_materialized(routes);
    assert_projected_rows_cover_route_receipts("live-flight-index", projected_rows, routes);
    assert_projected_rows_carry_algorithm_receipts("live-flight-index", projected_rows);
    assert_live_trace_contains_expected_source_fragments(&trace, &expected_source_fragments);
}

fn search_strategy_flow_live_replay_search_root() -> PathBuf {
    match env::var_os("PRJ_ROOT") {
        Some(root) => PathBuf::from(root),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap_or_else(|| panic!("resolve repository root from Cargo manifest"))
            .to_path_buf(),
    }
}

fn required_non_blank_env(name: &str) -> String {
    optional_non_blank_env(name).unwrap_or_else(|| {
        panic!("{name} must be set when {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV}=1")
    })
}

fn optional_non_blank_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn live_flight_timeout_seconds() -> u64 {
    optional_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_TIMEOUT_SECONDS_ENV)
        .map_or(
            WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_TIMEOUT_SECONDS,
            |raw| {
            raw.parse::<u64>()
                .unwrap_or_else(|error| panic!("parse {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_TIMEOUT_SECONDS_ENV}: {error}"))
                .max(1)
            },
        )
}

fn live_flight_expected_source_fragments() -> Vec<String> {
    optional_non_blank_env(
        WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_EXPECTED_SOURCE_FRAGMENTS_ENV,
    )
    .map(|raw| {
        raw.split(',')
            .map(str::trim)
            .filter(|fragment| !fragment.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    })
    .filter(|fragments| !fragments.is_empty())
    .unwrap_or_else(|| {
        vec![
            "packages/rust/crates/xiuxian-wendao".to_owned(),
            "wendao.toml".to_owned(),
            "packages/python".to_owned(),
            ".data/WendaoGraph.jl".to_owned(),
        ]
    })
}

fn assert_search_strategy_flow_live_replay_family(
    family: &'static str,
    intent: &str,
    expected_source_prefix: &str,
    search_root: &Path,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
) -> SearchStrategyFlowLiveReplayFamilyReport {
    let input_candidate_count = candidate_batch.row_count;
    let trace = run_wendaograph_search_strategy_flow_json_with_candidate_batch(
        intent,
        search_root,
        candidate_batch,
    )
    .unwrap_or_else(|error| panic!("run live SearchStrategyFlow replay for {family}: {error}"));
    assert_search_strategy_flow_live_replay_trace(
        family,
        expected_source_prefix,
        surface_markdown_file_count,
        surface_heading_count,
        input_candidate_count,
        &trace,
    )
}

fn assert_search_strategy_flow_live_replay_trace(
    family: &'static str,
    expected_source_prefix: &str,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    input_candidate_count: usize,
    trace: &str,
) -> SearchStrategyFlowLiveReplayFamilyReport {
    assert_search_strategy_flow_live_replay_trace_with_candidate_source(
        family,
        "rust-markdown-headings",
        true,
        true,
        expected_source_prefix,
        surface_markdown_file_count,
        surface_heading_count,
        input_candidate_count,
        trace,
    )
}

fn assert_search_strategy_flow_live_replay_trace_with_candidate_source(
    family: &'static str,
    expected_candidate_source: &str,
    require_selected_context_reduced: bool,
    require_stop_planner_action: bool,
    expected_source_prefix: &str,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    input_candidate_count: usize,
    trace: &str,
) -> SearchStrategyFlowLiveReplayFamilyReport {
    let trace: serde_json::Value = serde_json::from_str(&trace).unwrap_or_else(|error| {
        panic!("parse live SearchStrategyFlow replay for {family}: {error}")
    });
    assert_eq!(
        trace.get("candidateInputSource"),
        Some(&serde_json::json!(expected_candidate_source)),
        "{family} must use the expected Rust candidate bridge"
    );
    assert!(
        trace
            .get("candidateInputCount")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "{family} must discover local Markdown candidates"
    );
    assert_eq!(
        trace
            .get("candidateInputCount")
            .and_then(serde_json::Value::as_u64),
        Some(input_candidate_count as u64),
        "{family} trace must preserve bounded input candidate count"
    );

    let candidates = json_array(&trace, "candidates", family);
    let frontier = json_array(&trace, "frontier", family);
    let planner_actions = json_array(&trace, "plannerActions", family);
    let routes = trace
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{family} retrievalRoutes must be an array"));
    let projected_rows = trace
        .get("rustProjectedEvidenceRows")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{family} rustProjectedEvidenceRows must be an array"));
    assert!(!routes.is_empty(), "{family} must plan retrieval routes");
    assert!(
        !projected_rows.is_empty(),
        "{family} must project Rust evidence rows"
    );
    assert_summary_matches_candidate_rows(family, &trace, candidates);
    assert_strategy_flow_validation_flags(family, &trace, require_selected_context_reduced);
    assert_selected_frontier_and_planner_actions(
        family,
        frontier,
        planner_actions,
        require_stop_planner_action,
    );
    assert!(
        routes.iter().any(|route| {
            route
                .get("sourcePath")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with(expected_source_prefix))
        }),
        "{family} must route at least one {expected_source_prefix} candidate"
    );
    assert!(
        projected_rows.iter().any(|row| {
            row.get("sourcePath")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with(expected_source_prefix))
        }),
        "{family} must project at least one {expected_source_prefix} evidence row"
    );
    assert_projected_rows_cover_route_receipts(family, projected_rows, routes);
    assert_projected_rows_carry_algorithm_receipts(family, projected_rows);

    let selected_frontier_count = frontier
        .iter()
        .filter(|row| row.get("selected").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    let non_stop_planner_action_count = planner_actions
        .iter()
        .filter(|row| {
            row.get("actionKind")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| kind != "stop")
        })
        .count();

    SearchStrategyFlowLiveReplayFamilyReport {
        family,
        surface_markdown_file_count,
        surface_heading_count,
        input_candidate_count,
        trace_candidate_count: candidates.len(),
        frontier_count: frontier.len(),
        selected_frontier_count,
        planner_action_count: planner_actions.len(),
        non_stop_planner_action_count,
        route_count: routes.len(),
        projected_row_count: projected_rows.len(),
    }
}

fn configured_markdown_replay_inputs(
    search_root: &Path,
    max_candidates_per_family: Option<usize>,
) -> Vec<SearchStrategyFlowConfiguredMarkdownReplayInput> {
    search_strategy_flow_configured_markdown_replay_specs()
        .into_iter()
        .map(|spec| {
            let families = match max_candidates_per_family {
                Some(limit) => {
                    configured_wendaograph_search_strategy_flow_markdown_replay_families_with_limit(
                        search_root,
                        spec.intent,
                        limit,
                    )
                }
                None => configured_wendaograph_search_strategy_flow_markdown_replay_families(
                    search_root,
                    spec.intent,
                ),
            }
            .unwrap_or_else(|error| {
                panic!(
                    "build configured replay families for {}: {error}",
                    spec.family
                )
            });
            let family = families
                .into_iter()
                .find(|family| family.include_dir == spec.include_dir)
                .unwrap_or_else(|| {
                    panic!(
                        "configured Markdown replay family `{}` must exist for {}",
                        spec.include_dir, spec.family
                    )
                });
            SearchStrategyFlowConfiguredMarkdownReplayInput {
                spec,
                surface_markdown_file_count: family.markdown_file_count,
                surface_heading_count: family.heading_count,
                batch: family.batch,
            }
        })
        .collect()
}

fn run_configured_markdown_live_replay_reports(
    search_root: &Path,
    max_candidates_per_family: Option<usize>,
) -> SearchStrategyFlowLiveReplayRunReport {
    let started = Instant::now();
    let reports = configured_markdown_replay_inputs(search_root, max_candidates_per_family)
        .into_iter()
        .map(|input| {
            assert_search_strategy_flow_live_replay_family(
                input.spec.family,
                input.spec.intent,
                input.spec.expected_source_prefix,
                search_root,
                input.surface_markdown_file_count,
                input.surface_heading_count,
                input.batch,
            )
        })
        .collect::<Vec<_>>();
    let elapsed_ms = started.elapsed().as_millis();
    print_configured_markdown_live_replay_reports(max_candidates_per_family, &reports, elapsed_ms);
    SearchStrategyFlowLiveReplayRunReport {
        family_reports: reports,
        elapsed_ms,
    }
}

fn assert_configured_markdown_batch_replay_traces(
    replay_inputs: &[SearchStrategyFlowConfiguredMarkdownReplayInput],
    traces: &[String],
) -> Vec<SearchStrategyFlowLiveReplayFamilyReport> {
    assert_eq!(
        traces.len(),
        replay_inputs.len(),
        "batch replay must return one trace per configured Markdown family"
    );
    replay_inputs
        .iter()
        .zip(traces.iter())
        .map(|(input, trace)| {
            assert_search_strategy_flow_live_replay_trace(
                input.spec.family,
                input.spec.expected_source_prefix,
                input.surface_markdown_file_count,
                input.surface_heading_count,
                input.batch.row_count,
                trace,
            )
        })
        .collect()
}

fn assert_configured_markdown_live_replay_reports(
    reports: &[SearchStrategyFlowLiveReplayFamilyReport],
) {
    assert_eq!(reports.len(), 4, "configured Markdown replay family count");
    assert!(
        reports
            .iter()
            .all(|report| report.input_candidate_count == report.trace_candidate_count),
        "trace candidates must match bounded input candidate counts"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.input_candidate_count <= 12),
        "each configured family must keep bounded top-N replay input rows"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.surface_markdown_file_count > 0),
        "each replay family must keep its surface Markdown file count"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.surface_heading_count >= report.surface_markdown_file_count),
        "each replay family must keep heading evidence"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.selected_frontier_count > 0),
        "each replay family must select frontier rows"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.non_stop_planner_action_count > 0),
        "each replay family must emit non-stop planner actions"
    );
    assert!(
        reports.iter().all(|report| report.route_count > 0),
        "each replay family must plan routes"
    );
    assert!(
        reports.iter().all(|report| report.projected_row_count > 0),
        "each replay family must project Rust evidence rows"
    );

    let surface_markdown_files = reports
        .iter()
        .map(|report| report.surface_markdown_file_count)
        .sum::<usize>();
    let surface_headings = reports
        .iter()
        .map(|report| report.surface_heading_count)
        .sum::<usize>();
    let input_candidates = reports
        .iter()
        .map(|report| report.input_candidate_count)
        .sum::<usize>();
    let selected_frontier = reports
        .iter()
        .map(|report| report.selected_frontier_count)
        .sum::<usize>();
    let frontier_rows = reports
        .iter()
        .map(|report| report.frontier_count)
        .sum::<usize>();
    let planner_actions = reports
        .iter()
        .map(|report| report.planner_action_count)
        .sum::<usize>();
    let routes = reports
        .iter()
        .map(|report| report.route_count)
        .sum::<usize>();
    let projected_rows = reports
        .iter()
        .map(|report| report.projected_row_count)
        .sum::<usize>();
    assert!(surface_markdown_files >= 400);
    assert!(surface_headings >= surface_markdown_files);
    assert_eq!(
        input_candidates, 48,
        "current bounded configured Markdown replay should feed twelve rows per family"
    );
    assert!(selected_frontier >= reports.len());
    assert!(frontier_rows >= selected_frontier);
    assert!(planner_actions >= reports.len() * 2);
    assert!(routes >= reports.len());
    assert!(projected_rows >= reports.len());
}

fn print_configured_markdown_live_replay_reports(
    max_candidates_per_family: Option<usize>,
    reports: &[SearchStrategyFlowLiveReplayFamilyReport],
    elapsed_ms: u128,
) {
    let surface_markdown_files = reports
        .iter()
        .map(|report| report.surface_markdown_file_count)
        .sum::<usize>();
    let surface_headings = reports
        .iter()
        .map(|report| report.surface_heading_count)
        .sum::<usize>();
    let input_candidates = reports
        .iter()
        .map(|report| report.input_candidate_count)
        .sum::<usize>();
    let frontier_rows = reports
        .iter()
        .map(|report| report.frontier_count)
        .sum::<usize>();
    let selected_frontier = reports
        .iter()
        .map(|report| report.selected_frontier_count)
        .sum::<usize>();
    let planner_actions = reports
        .iter()
        .map(|report| report.planner_action_count)
        .sum::<usize>();
    let routes = reports
        .iter()
        .map(|report| report.route_count)
        .sum::<usize>();
    let projected_rows = reports
        .iter()
        .map(|report| report.projected_row_count)
        .sum::<usize>();
    eprintln!(
        "SearchStrategyFlow configured Markdown replay summary: candidateBudget={}, families={}, surfaceMarkdownFiles={}, surfaceHeadings={}, boundedInputCandidates={}, frontierRows={}, selectedFrontier={}, plannerActions={}, routes={}, projectedRows={}, elapsedMs={}",
        max_candidates_per_family.map_or("default".to_owned(), |limit| limit.to_string()),
        reports.len(),
        surface_markdown_files,
        surface_headings,
        input_candidates,
        frontier_rows,
        selected_frontier,
        planner_actions,
        routes,
        projected_rows,
        elapsed_ms
    );
    for report in reports {
        eprintln!(
            "SearchStrategyFlow configured Markdown family report: family={}, surfaceMarkdownFiles={}, surfaceHeadings={}, boundedInputCandidates={}, selectedFrontier={}, plannerActions={}, routes={}, projectedRows={}",
            report.family,
            report.surface_markdown_file_count,
            report.surface_heading_count,
            report.input_candidate_count,
            report.selected_frontier_count,
            report.planner_action_count,
            report.route_count,
            report.projected_row_count
        );
    }
}

fn assert_live_flight_routes_materialized(routes: &[serde_json::Value]) {
    assert!(
        routes.iter().any(|route| {
            route
                .get("materializationStatus")
                .and_then(serde_json::Value::as_str)
                == Some("executed")
                && route
                    .get("decodedPayloadStatus")
                    .and_then(serde_json::Value::as_str)
                    == Some("decoded")
                && route
                    .get("materializedRows")
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|row_count| row_count > 0)
        }),
        "live Flight replay must execute and decode at least one route"
    );
    for route in routes.iter().filter(|route| {
        route
            .get("materializationStatus")
            .and_then(serde_json::Value::as_str)
            == Some("executed")
    }) {
        assert_eq!(
            route
                .get("decodedPayloadStatus")
                .and_then(serde_json::Value::as_str),
            Some("decoded"),
            "executed live Flight routes must decode payloads"
        );
        assert!(
            route
                .get("routeReceipts")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|receipts| !receipts.is_empty()),
            "executed live Flight routes must keep route receipts"
        );
        assert!(
            route
                .get("decodedPayloadReceipts")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|receipts| !receipts.is_empty()),
            "executed live Flight routes must keep decoded payload receipts"
        );
    }
}

fn assert_live_trace_contains_expected_source_fragments(
    trace: &serde_json::Value,
    expected_source_fragments: &[String],
) {
    let trace_text = serde_json::to_string(trace)
        .unwrap_or_else(|error| panic!("serialize live Flight trace for assertions: {error}"));
    for expected_fragment in expected_source_fragments {
        assert!(
            trace_text.contains(expected_fragment),
            "live Flight trace must include expected source fragment `{expected_fragment}`"
        );
    }
}

fn json_array<'a>(
    trace: &'a serde_json::Value,
    field: &str,
    family: &str,
) -> &'a Vec<serde_json::Value> {
    trace
        .get(field)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{family} {field} must be an array"))
}

fn assert_summary_matches_candidate_rows(
    family: &str,
    trace: &serde_json::Value,
    candidates: &[serde_json::Value],
) {
    assert_eq!(
        trace
            .get("summary")
            .and_then(|summary| summary.get("candidateCount"))
            .and_then(serde_json::Value::as_u64),
        Some(candidates.len() as u64),
        "{family} summary must count replayed candidate rows"
    );
}

fn assert_strategy_flow_validation_flags(
    family: &str,
    trace: &serde_json::Value,
    require_selected_context_reduced: bool,
) {
    let validation = trace
        .get("validation")
        .unwrap_or_else(|| panic!("{family} validation object must exist"));
    for flag in ["noVectorMode", "blockedEvidencePruned"] {
        assert_eq!(
            validation.get(flag).and_then(serde_json::Value::as_bool),
            Some(true),
            "{family} validation.{flag} must be true"
        );
    }
    if require_selected_context_reduced {
        assert_eq!(
            validation
                .get("selectedContextReduced")
                .and_then(serde_json::Value::as_bool),
            Some(true),
            "{family} validation.selectedContextReduced must be true"
        );
    }
}

fn assert_selected_frontier_and_planner_actions(
    family: &str,
    frontier: &[serde_json::Value],
    planner_actions: &[serde_json::Value],
    require_stop_planner_action: bool,
) {
    assert!(
        frontier
            .iter()
            .any(|row| row.get("selected").and_then(serde_json::Value::as_bool) == Some(true)),
        "{family} must select at least one frontier branch"
    );
    assert!(
        planner_actions.iter().any(|row| {
            row.get("actionKind")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| kind != "stop")
        }),
        "{family} must emit at least one non-stop planner action"
    );
    if require_stop_planner_action {
        assert!(
            planner_actions
                .iter()
                .any(
                    |row| row.get("actionKind").and_then(serde_json::Value::as_str) == Some("stop")
                ),
            "{family} must preserve a stop planner action for bounded replay"
        );
    }
}

fn assert_projected_rows_cover_route_receipts(
    family: &str,
    projected_rows: &[serde_json::Value],
    routes: &[serde_json::Value],
) {
    let planned_route_ids = routes
        .iter()
        .filter_map(|row| row.get("candidateId").and_then(serde_json::Value::as_str))
        .collect::<HashSet<_>>();
    let projected_route_ids = projected_rows
        .iter()
        .filter(|row| row.get("routePlanned").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|row| row.get("candidateId").and_then(serde_json::Value::as_str))
        .collect::<HashSet<_>>();

    assert!(
        !planned_route_ids.is_empty(),
        "{family} must have planned route candidate ids"
    );
    assert!(
        planned_route_ids.is_subset(&projected_route_ids),
        "{family} projected rows must mark every planned route"
    );
}

fn assert_projected_rows_carry_algorithm_receipts(
    family: &str,
    projected_rows: &[serde_json::Value],
) {
    assert!(
        projected_rows.iter().all(|row| {
            row.get("projectionSource")
                .and_then(serde_json::Value::as_str)
                == Some("rust-bridge-search-strategy-flow-v1")
        }),
        "{family} projected rows must use the Rust bridge projection source"
    );
    assert!(
        projected_rows.iter().any(|row| {
            row.get("selected").and_then(serde_json::Value::as_bool) == Some(true)
                && row
                    .get("proofTags")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|tags| tags.contains(&serde_json::json!("frontier_selected")))
        }),
        "{family} projected rows must carry selected-frontier proof tags"
    );
    assert!(
        projected_rows.iter().any(|row| {
            row.get("proofTags")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|tags| {
                    tags.contains(&serde_json::json!("rust_projected"))
                        && tags.contains(&serde_json::json!("real_document"))
                        && tags.contains(&serde_json::json!("route_planned"))
                })
        }),
        "{family} projected rows must carry Rust, real-document, and route tags"
    );
}

#[tokio::test]
async fn search_strategy_flow_flight_candidate_discovery_decodes_non_markdown_source_config_rows() {
    let (base_url, server) = spawn_fake_search_strategy_flow_candidate_discovery_service().await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight candidate config: {error}"));

    let batch = search_strategy_flow_candidate_input_batch_from_repo_search(
        "search strategy flow link graph python julia toml",
        &config,
    )
    .await
    .unwrap_or_else(|error| panic!("discover fake Flight candidates: {error}"));
    server.abort();

    assert_eq!(batch.source, "rust-flight-repo-search");
    assert_eq!(batch.row_count, 4);
    assert_eq!(batch.tsv.lines().count(), 4);
    for expected_fragment in [
        "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
        "wendao.toml",
        "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
        ".data/WendaoGraph.jl/src/reasoning/search_strategy_flow/frontier.jl",
        "ppr-runtime-search-strategy",
        "wendao-repository-configuration",
        "python-analyzer-worker",
        "julia-frontier-strategy",
        "effective-parser:rust-lang-parser",
        "effective-parser:xiuxian-ast:toml",
        "effective-parser:xiuxian-ast:python",
        "effective-parser:julia-lang-parser",
        "parser-priority:local-override",
        "parser-priority:general-baseline",
        "repo-search",
        "arrow-flight",
    ] {
        assert!(
            batch.tsv.contains(expected_fragment),
            "candidate batch should contain `{expected_fragment}`:\n{}",
            batch.tsv
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "JSON bridge fixture is intentionally explicit"
)]
fn search_strategy_flow_rust_bridge_adds_planned_retrieval_routes() {
    let trace = serde_json::json!({
        "intent": "find query understanding",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "action": "keep",
                "reason": "score",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            },
            {
                "candidateId": "docs/90_validation/90.01_validation.md#promotion-boundary",
                "action": "prune",
                "reason": "blocked",
                "finalScore": 0.2,
                "evidenceCoverage": 0.1,
                "graphScore": 0.1,
                "authorityScore": 0.1,
                "semanticScore": 0.0,
                "structuralScore": 0.1,
                "contextCost": 100,
                "blocked": true
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "rank": 1,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });

    let enriched = enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace.to_string())
        .unwrap_or_else(|error| panic!("enrich SearchStrategyFlow bridge trace: {error}"));
    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse enriched SearchStrategyFlow bridge trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    let projected_rows = enriched
        .get("rustProjectedEvidenceRows")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("rustProjectedEvidenceRows must be an array"));

    assert_eq!(routes.len(), 1);
    assert_eq!(projected_rows.len(), 2);
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("planned"))
    );
    assert_eq!(
        route.get("receiptSource"),
        Some(&serde_json::json!("rust-bridge"))
    );
    assert_eq!(
        route.get("sourcePath"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        route.get("headingAnchor"),
        Some(&serde_json::json!("stage-1-query-understanding"))
    );
    assert!(route.get("materializedRows").is_none());
    assert!(route.get("routeReceipts").is_none());
    assert_eq!(
        route
            .get("flightSteps")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    let graph_step = route
        .get("flightSteps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.last())
        .unwrap_or_else(|| panic!("graph flight step"));
    assert_eq!(
        graph_step
            .get("requiresResolvedGraphNodeId")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        serde_json::to_string(graph_step)
            .unwrap_or_else(|error| panic!("graph flight step should serialize: {error}"))
            .contains("<resolved-graph-node-id>")
    );

    let selected_evidence = projected_rows
        .iter()
        .find(|row| {
            row.get("candidateId")
                == Some(&serde_json::json!(
                    "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding"
                ))
        })
        .unwrap_or_else(|| panic!("selected projected evidence row"));
    assert_eq!(
        selected_evidence.get("projectionSource"),
        Some(&serde_json::json!("rust-bridge-search-strategy-flow-v1"))
    );
    assert_eq!(
        selected_evidence.get("sourcePath"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        selected_evidence.get("headingAnchor"),
        Some(&serde_json::json!("stage-1-query-understanding"))
    );
    assert_eq!(
        selected_evidence.get("evidenceKind"),
        Some(&serde_json::json!("search_strategy_flow_authority"))
    );
    assert_eq!(
        selected_evidence.get("selected"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        selected_evidence.get("plannerMaterialized"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        selected_evidence.get("retrievalRouteCount"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        selected_evidence.get("routePlanned"),
        Some(&serde_json::json!(true))
    );
    assert!(
        selected_evidence
            .get("proofTags")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("proofTags must be an array"))
            .contains(&serde_json::json!("route_planned"))
    );

    let blocked_evidence = projected_rows
        .iter()
        .find(|row| {
            row.get("candidateId")
                == Some(&serde_json::json!(
                    "docs/90_validation/90.01_validation.md#promotion-boundary"
                ))
        })
        .unwrap_or_else(|| panic!("blocked projected evidence row"));
    assert_eq!(
        blocked_evidence.get("blocked"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        blocked_evidence.get("selected"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        blocked_evidence.get("routePlanned"),
        Some(&serde_json::json!(false))
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "JSON bridge fixture is intentionally explicit"
)]
fn search_strategy_flow_rust_bridge_requires_section_granularity() {
    let trace = serde_json::json!({
        "intent": "find SearchStrategyFlow precision pruning",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "action": "keep",
                "reason": "file-level candidate should not materialize",
                "finalScore": 0.93,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            },
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "action": "keep",
                "reason": "section-level candidate should materialize",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 800,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "rank": 1,
                "selected": true,
                "finalScore": 0.93,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            },
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "rank": 2,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 800,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.93,
                "contextBudget": 1000,
                "reason": "file_level"
            },
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 800,
                "reason": "section_level"
            }
        ],
        "summary": {},
        "validation": {}
    });

    let enriched = enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace.to_string())
        .unwrap_or_else(|error| panic!("enrich SearchStrategyFlow bridge trace: {error}"));
    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse enriched SearchStrategyFlow bridge trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));

    assert_eq!(routes.len(), 1);
    assert_eq!(
        routes[0].get("candidateId"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.02_precision_pruning.md#precision-score"
        ))
    );
    assert_eq!(
        routes[0].get("headingAnchor"),
        Some(&serde_json::json!("precision-score"))
    );
    assert_eq!(
        routes[0].get("directFileReadAllowed"),
        Some(&serde_json::json!(false))
    );
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight materialization fixture documents every decoded receipt"
)]
async fn search_strategy_flow_flight_materialization_executes_and_decodes_route_receipts() {
    let trace = serde_json::json!({
        "intent": "find query understanding",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "action": "keep",
                "reason": "score",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "rank": 1,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let (base_url, server) = spawn_fake_search_strategy_flow_flight_service().await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute fake Flight materialization: {error}"));
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    assert_eq!(routes.len(), 1);
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("decodedPayloadStatus"),
        Some(&serde_json::json!("decoded"))
    );
    assert_eq!(route.get("materializedRows"), Some(&serde_json::json!(4)));
    assert_eq!(
        route.get("resolvedNodeId"),
        Some(&serde_json::json!("node:stage-1-query-understanding"))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );

    let route_receipts = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("routeReceipts must be an array"));
    let decoded_receipts = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("decodedPayloadReceipts must be an array"));
    assert_eq!(route_receipts.len(), 4);
    assert_eq!(decoded_receipts.len(), 4);
    for expected_route in [
        REPO_SEARCH_ROUTE,
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        GRAPH_NEIGHBORS_ROUTE,
    ] {
        assert!(
            route_receipts.iter().any(|receipt| {
                receipt.get("route").and_then(serde_json::Value::as_str) == Some(expected_route)
                    && receipt.get("rowCount").and_then(serde_json::Value::as_u64) == Some(1)
            }),
            "route receipt for {expected_route} should exist"
        );
        assert!(
            decoded_receipts.iter().any(|receipt| {
                receipt.get("route").and_then(serde_json::Value::as_str) == Some(expected_route)
                    && receipt.get("rowCount").and_then(serde_json::Value::as_u64) == Some(1)
            }),
            "decoded receipt for {expected_route} should exist"
        );
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight reference fixture documents the non-Markdown route contract"
)]
async fn search_strategy_flow_flight_materialization_executes_non_markdown_reference_route() {
    let scenario = SearchStrategyFlowFakeFlightScenario::rust_reference();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find rust ppr search strategy",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": candidate_id.clone(),
                "action": "keep",
                "reason": "score",
                "finalScore": 0.92,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": candidate_id.clone(),
                "rank": 1,
                "selected": true,
                "finalScore": 0.92,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": candidate_id.clone(),
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.92,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let (base_url, server) = spawn_fake_search_strategy_flow_flight_service_for(scenario).await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute fake Flight materialization: {error}"));
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    assert_eq!(routes.len(), 1);
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("decodedPayloadStatus"),
        Some(&serde_json::json!("decoded"))
    );
    assert_eq!(
        route.get("sourcePath"),
        Some(&serde_json::json!(scenario.source_path))
    );
    assert_eq!(
        route.get("resolvedPageId"),
        Some(&serde_json::json!(scenario.page_id))
    );
    assert!(
        route
            .get("resolvedPageId")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|page_id| page_id.contains(":projection:reference:")),
        "non-Markdown routes must use reference projection semantics"
    );
    assert_eq!(
        route.get("resolvedNodeId"),
        Some(&serde_json::json!(scenario.node_id))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(format!("docs/{}", scenario.source_path)))
    );

    let decoded_receipts = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("decodedPayloadReceipts must be an array"));
    assert!(decoded_receipts.iter().any(|receipt| {
        receipt.get("route").and_then(serde_json::Value::as_str) == Some(REPO_SEARCH_ROUTE)
            && receipt
                .get("evidenceAnchor")
                .and_then(serde_json::Value::as_str)
                == Some(format!("path:{}", scenario.source_path).as_str())
    }));
    assert!(decoded_receipts.iter().any(|receipt| {
        receipt.get("route").and_then(serde_json::Value::as_str)
            == Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE)
            && receipt
                .get("evidenceAnchor")
                .and_then(serde_json::Value::as_str)
                == Some(scenario.node_id)
    }));
}

#[test]
fn search_strategy_flow_probe_actions_are_whitelisted() {
    assert_eq!(
        search_strategy_flow_probe_action_route("expand_neighbors"),
        Ok(Some(GRAPH_NEIGHBORS_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("expand_neighbors:docs-fixture/docs/search.md"),
        Ok(Some(GRAPH_NEIGHBORS_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("open_parent_child"),
        Ok(Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("compare_provenance"),
        Ok(Some(REPO_SEARCH_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("open_adjacent_sections"),
        Ok(Some(ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE))
    );
    assert_eq!(search_strategy_flow_probe_action_route("stop"), Ok(None));
    assert!(parse_search_strategy_flow_probe_action("open_full_file").is_err());
}

#[tokio::test]
async fn search_strategy_flow_rust_bridge_rejects_invalid_flight_endpoint_before_execution() {
    let trace = serde_json::json!({
        "intent": "find query understanding",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "action": "keep",
                "reason": "score",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "rank": 1,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let config = SearchStrategyFlowFlightMaterializationConfig::new("not a url", "docs")
        .unwrap_or_else(|error| panic!("create Flight materialization config: {error}"));

    let error =
        match enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        {
            Ok(trace) => panic!(
                "invalid endpoint should reject before executed receipts are fabricated, got {trace}"
            ),
            Err(error) => error,
        };

    assert!(error.contains("create SearchStrategyFlow Flight endpoint"));
}
