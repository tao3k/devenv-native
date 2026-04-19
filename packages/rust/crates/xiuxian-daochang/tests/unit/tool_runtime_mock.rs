use std::future::{Future, pending};
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use xiuxian_daochang::{ToolRuntimeCallResult, ToolRuntimeToolDefinition};

const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";
const JSON_RPC_VERSION: &str = "2.0";
const SESSION_HEADER: &str = "Mcp-Session-Id";
const DEFAULT_SESSION_ID: &str = "mock-tool-runtime-session";

#[derive(Clone, Debug, Default)]
pub(crate) struct MockListToolsRequest;

#[derive(Clone, Debug)]
pub(crate) struct MockToolCall {
    pub(crate) name: String,
    pub(crate) arguments: Option<Map<String, Value>>,
}

pub(crate) enum MockListToolsReply {
    Result(Vec<ToolRuntimeToolDefinition>),
    Hang,
}

pub(crate) enum MockCallToolReply {
    Result(ToolRuntimeCallResult),
    RpcError {
        code: i64,
        message: String,
        data: Option<Value>,
    },
    Hang,
}

pub(crate) type ListToolsHandler =
    Arc<dyn Fn(MockListToolsRequest) -> BoxFuture<'static, MockListToolsReply> + Send + Sync>;
pub(crate) type CallToolHandler =
    Arc<dyn Fn(MockToolCall) -> BoxFuture<'static, MockCallToolReply> + Send + Sync>;

#[derive(Clone)]
pub(crate) struct MockToolRuntimeConfig {
    session_id: String,
    list_tools: ListToolsHandler,
    call_tool: CallToolHandler,
}

impl MockToolRuntimeConfig {
    pub(crate) fn with_handlers(list_tools: ListToolsHandler, call_tool: CallToolHandler) -> Self {
        Self {
            session_id: DEFAULT_SESSION_ID.to_string(),
            list_tools,
            call_tool,
        }
    }

    pub(crate) fn with_static_tools(
        tools: Vec<ToolRuntimeToolDefinition>,
        call_tool: CallToolHandler,
    ) -> Self {
        let list_tools = list_handler(move |_request| {
            let tools = tools.clone();
            async move { MockListToolsReply::Result(tools) }
        });
        Self::with_handlers(list_tools, call_tool)
    }
}

pub(crate) fn list_handler<F, Fut>(handler: F) -> ListToolsHandler
where
    F: Fn(MockListToolsRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = MockListToolsReply> + Send + 'static,
{
    Arc::new(move |request| Box::pin(handler(request)))
}

pub(crate) fn call_handler<F, Fut>(handler: F) -> CallToolHandler
where
    F: Fn(MockToolCall) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = MockCallToolReply> + Send + 'static,
{
    Arc::new(move |request| Box::pin(handler(request)))
}

pub(crate) fn permissive_tool_definition(
    name: &str,
    description: &str,
) -> ToolRuntimeToolDefinition {
    tool_definition(
        name,
        description,
        json!({
            "type": "object",
            "additionalProperties": true
        }),
    )
}

pub(crate) fn tool_definition(
    name: &str,
    description: &str,
    input_schema: Value,
) -> ToolRuntimeToolDefinition {
    ToolRuntimeToolDefinition {
        name: name.to_string(),
        description: Some(description.to_string()),
        input_schema: input_schema.as_object().cloned().unwrap_or_default(),
    }
}

pub(crate) fn text_result(text: impl Into<String>) -> ToolRuntimeCallResult {
    ToolRuntimeCallResult {
        text_segments: vec![text.into()],
        is_error: false,
    }
}

pub(crate) async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
    drop(probe);
    addr
}

pub(crate) async fn spawn_mock_tool_runtime(
    addr: std::net::SocketAddr,
    config: MockToolRuntimeConfig,
) -> tokio::task::JoinHandle<()> {
    let app = Router::new()
        .route("/", post(handle_rpc))
        .route("/sse", post(handle_rpc))
        .with_state(Arc::new(config));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind mock tool listener");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Deserialize, Default)]
struct ToolRuntimeListParams {
    #[serde(default)]
    cursor: Option<String>,
}

#[derive(Deserialize)]
struct ToolRuntimeCallParams {
    name: String,
    #[serde(default)]
    arguments: Option<Map<String, Value>>,
}

#[derive(Serialize)]
struct JsonRpcSuccessResponse<T> {
    jsonrpc: &'static str,
    id: Value,
    result: T,
}

#[derive(Serialize)]
struct JsonRpcErrorResponse {
    jsonrpc: &'static str,
    id: Value,
    error: JsonRpcError,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeInitializeResult {
    protocol_version: &'static str,
    capabilities: Value,
    server_info: ToolRuntimeImplementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeImplementation {
    name: String,
    title: Option<String>,
    version: String,
    description: Option<String>,
    icons: Option<Vec<Value>>,
    website_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeCallWireResult {
    content: Vec<ToolRuntimeContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Serialize)]
struct ToolRuntimeContentBlock {
    #[serde(rename = "type")]
    kind: &'static str,
    text: String,
}

async fn handle_rpc(
    State(config): State<Arc<MockToolRuntimeConfig>>,
    _headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let response = match request.method.as_str() {
        "initialize" => initialize_response(request.id),
        "notifications/initialized" => StatusCode::ACCEPTED.into_response(),
        "tools/list" => {
            let params = request
                .params
                .and_then(|value| serde_json::from_value::<ToolRuntimeListParams>(value).ok())
                .unwrap_or_default();
            let _ = params.cursor;
            match (config.list_tools)(MockListToolsRequest).await {
                MockListToolsReply::Result(tools) => json_rpc_success(
                    request.id,
                    serde_json::json!({
                        "tools": tools,
                    }),
                ),
                MockListToolsReply::Hang => pending::<Response>().await,
            }
        }
        "tools/call" => {
            let params = request
                .params
                .and_then(|value| serde_json::from_value::<ToolRuntimeCallParams>(value).ok());
            let Some(params) = params else {
                return with_session_header(
                    json_rpc_error(request.id, -32602, "invalid tools/call params", None),
                    &config.session_id,
                );
            };
            match (config.call_tool)(MockToolCall {
                name: params.name,
                arguments: params.arguments,
            })
            .await
            {
                MockCallToolReply::Result(result) => json_rpc_success(
                    request.id,
                    ToolRuntimeCallWireResult {
                        content: result
                            .text_segments
                            .into_iter()
                            .map(|text| ToolRuntimeContentBlock { kind: "text", text })
                            .collect(),
                        is_error: Some(result.is_error),
                    },
                ),
                MockCallToolReply::RpcError {
                    code,
                    message,
                    data,
                } => json_rpc_error(request.id, code, message, data),
                MockCallToolReply::Hang => pending::<Response>().await,
            }
        }
        other => json_rpc_error(
            request.id,
            -32601,
            format!("unsupported method: {other}"),
            None,
        ),
    };
    with_session_header(response, &config.session_id)
}

fn initialize_response(id: Option<Value>) -> Response {
    json_rpc_success(
        id,
        ToolRuntimeInitializeResult {
            protocol_version: DEFAULT_PROTOCOL_VERSION,
            capabilities: json!({
                "tools": {}
            }),
            server_info: ToolRuntimeImplementation {
                name: "mock-tool-runtime".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some("test-only tool runtime".to_string()),
                icons: None,
                website_url: None,
            },
            instructions: None,
        },
    )
}

fn json_rpc_success<T>(id: Option<Value>, result: T) -> Response
where
    T: Serialize,
{
    Json(JsonRpcSuccessResponse {
        jsonrpc: JSON_RPC_VERSION,
        id: id.unwrap_or_else(|| json!(0)),
        result,
    })
    .into_response()
}

fn json_rpc_error(
    id: Option<Value>,
    code: i64,
    message: impl Into<String>,
    data: Option<Value>,
) -> Response {
    Json(JsonRpcErrorResponse {
        jsonrpc: JSON_RPC_VERSION,
        id: id.unwrap_or_else(|| json!(0)),
        error: JsonRpcError {
            code,
            message: message.into(),
            data,
        },
    })
    .into_response()
}

fn with_session_header(mut response: Response, session_id: &str) -> Response {
    if let Ok(value) = HeaderValue::from_str(session_id) {
        response.headers_mut().insert(SESSION_HEADER, value);
    }
    response
}
