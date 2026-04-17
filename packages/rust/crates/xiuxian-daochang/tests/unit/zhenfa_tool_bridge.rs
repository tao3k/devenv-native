//! End-to-end tests for `run_turn` tool dispatch integration.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde_json::{Value, json};
use xiuxian_daochang::{
    Agent, AgentConfig, NativeToolRegistry, SessionStore, ToolServerEntry, WendaoSearchTool,
    WendaoSearchToolConfig,
};
use xiuxian_qianji::BootcampLlmMode;

const AUTHOR_RESPONSE_XML: &str = r"
<sql_author_spec>
  <target_object>wendao_sql_tables</target_object>
  <projection>
    <column>sql_table_name</column>
  </projection>
  <order_by>
    <item>
      <column>sql_table_name</column>
      <direction>asc</direction>
    </item>
  </order_by>
  <limit>1</limit>
</sql_author_spec>
";

#[derive(Clone)]
struct MockAgentLlmScenario {
    tool_name: String,
    tool_arguments: String,
    final_response: String,
}

#[derive(Clone)]
struct MockAgentLlmState {
    round: Arc<AtomicUsize>,
    last_tool_message: Arc<std::sync::Mutex<Option<serde_json::Value>>>,
    scenario: MockAgentLlmScenario,
}

#[derive(Clone, Default)]
struct MockGatewayState {
    observed_queries: Arc<tokio::sync::Mutex<Vec<String>>>,
}

fn test_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

async fn mock_agent_llm_chat_handler(
    State(state): State<MockAgentLlmState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let round = state.round.fetch_add(1, Ordering::SeqCst);
    if round > 0 {
        let tool_message = payload
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .and_then(|messages| {
                messages.iter().find_map(|message| {
                    if message.get("role").and_then(serde_json::Value::as_str) != Some("tool") {
                        return None;
                    }
                    Some(message.clone())
                })
            });
        let mut guard = state
            .last_tool_message
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = tool_message;
    }

    let response = if round == 0 {
        json!({
            "id": "mock-chatcmpl-tool-call",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "finish_reason": "tool_calls",
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": state.scenario.tool_name,
                            "arguments": state.scenario.tool_arguments
                        }
                    }]
                }
            }]
        })
    } else {
        json!({
            "id": "mock-chatcmpl-final",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": state.scenario.final_response
                }
            }]
        })
    };
    Json(response)
}

fn extract_message_content(message: &serde_json::Value) -> String {
    if let Some(text) = message.get("content").and_then(serde_json::Value::as_str) {
        return text.to_string();
    }
    message
        .get("content")
        .map_or_else(String::new, serde_json::Value::to_string)
}

async fn reserve_local_addr() -> anyhow::Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = probe.local_addr()?;
    drop(probe);
    Ok(addr)
}

async fn spawn_mock_agent_llm_server(
    addr: std::net::SocketAddr,
    scenario: MockAgentLlmScenario,
) -> anyhow::Result<(
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Option<serde_json::Value>>>,
)> {
    let last_tool_message = Arc::new(std::sync::Mutex::new(None));
    let state = MockAgentLlmState {
        round: Arc::new(AtomicUsize::new(0)),
        last_tool_message: Arc::clone(&last_tool_message),
        scenario,
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(mock_agent_llm_chat_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        }),
        last_tool_message,
    ))
}

async fn spawn_mock_gateway(state: MockGatewayState) -> anyhow::Result<String> {
    let app = Router::new()
        .route("/query", post(mock_query_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|error| panic!("mock gateway should serve successfully: {error}"));
    });
    Ok(format!("http://{address}/query"))
}

async fn mock_query_handler(
    State(state): State<MockGatewayState>,
    Json(request): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let query = request
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    state.observed_queries.lock().await.push(query.clone());

    let payload = if query
        .contains("FROM wendao_sql_tables ORDER BY sql_table_name, COALESCE(repo_id, '')")
    {
        sql_response(
            &[
                json!({"name": "sql_table_name", "dataType": "Utf8", "nullable": false}),
                json!({"name": "corpus", "dataType": "Utf8", "nullable": false}),
                json!({"name": "scope", "dataType": "Utf8", "nullable": false}),
                json!({"name": "sql_object_kind", "dataType": "Utf8", "nullable": false}),
                json!({"name": "source_count", "dataType": "Int64", "nullable": false}),
                json!({"name": "repo_id", "dataType": "Utf8", "nullable": true}),
            ],
            &[json!({
                "sql_table_name": "wendao_sql_tables",
                "corpus": "catalog",
                "scope": "request",
                "sql_object_kind": "table",
                "source_count": 1,
                "repo_id": null
            })],
        )
    } else if query
        .contains("FROM wendao_sql_columns ORDER BY sql_table_name, ordinal_position, column_name")
    {
        sql_response(
            &[
                json!({"name": "sql_table_name", "dataType": "Utf8", "nullable": false}),
                json!({"name": "column_name", "dataType": "Utf8", "nullable": false}),
                json!({"name": "data_type", "dataType": "Utf8", "nullable": false}),
                json!({"name": "is_nullable", "dataType": "Boolean", "nullable": false}),
                json!({"name": "ordinal_position", "dataType": "Int64", "nullable": false}),
                json!({"name": "column_origin_kind", "dataType": "Utf8", "nullable": false}),
            ],
            &[json!({
                "sql_table_name": "wendao_sql_tables",
                "column_name": "sql_table_name",
                "data_type": "Utf8",
                "is_nullable": false,
                "ordinal_position": 1,
                "column_origin_kind": "physical"
            })],
        )
    } else if query
        == "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1"
    {
        sql_response(
            &[json!({"name": "sql_table_name", "dataType": "Utf8", "nullable": false})],
            &[json!({"sql_table_name": "wendao_sql_tables"})],
        )
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("unexpected query: {query}")
            })),
        );
    };

    (StatusCode::OK, Json(payload))
}

fn sql_response(columns: &[Value], rows: &[Value]) -> Value {
    json!({
        "query_language": "sql",
        "payload": {
            "metadata": {
                "catalogTableName": "wendao_sql_tables",
                "columnCatalogTableName": "wendao_sql_columns",
                "viewSourceCatalogTableName": "wendao_sql_view_sources",
                "supportsInformationSchema": true,
                "registeredTables": ["wendao_sql_tables", "wendao_sql_columns"],
                "registeredTableCount": 2,
                "registeredViewCount": 0,
                "registeredColumnCount": columns.len(),
                "registeredViewSourceCount": 0,
                "resultBatchCount": 1,
                "resultRowCount": rows.len(),
            },
            "batches": [{
                "rowCount": rows.len(),
                "columns": columns,
                "rows": rows,
            }]
        }
    })
}

#[tokio::test]
async fn run_turn_dispatches_wendao_search_through_native_tool() -> anyhow::Result<()> {
    let _guard = test_lock().lock().await;

    let project_root = tempfile::tempdir()?;
    let gateway_state = MockGatewayState::default();
    let gateway_endpoint = spawn_mock_gateway(gateway_state.clone()).await?;

    let agent_llm_addr = reserve_local_addr().await?;
    let scenario = MockAgentLlmScenario {
        tool_name: "wendao.search".to_string(),
        tool_arguments: "{\"request\":\"Show me the available SQL tables.\"}".to_string(),
        final_response: "final answer after native wendao search".to_string(),
    };
    let (agent_llm_server, last_tool_message) =
        spawn_mock_agent_llm_server(agent_llm_addr, scenario).await?;

    let mut native_tools = NativeToolRegistry::new();
    native_tools.register(Arc::new(WendaoSearchTool::new_with_llm_mode(
        WendaoSearchToolConfig::new(
            gateway_endpoint.clone(),
            Some(project_root.path().display().to_string()),
            HashMap::new(),
        ),
        BootcampLlmMode::Mock {
            response: AUTHOR_RESPONSE_XML.to_string(),
        },
    )));

    let agent = Agent::from_config_with_session_backends_and_native_tools_for_test(
        AgentConfig {
            inference_url: format!("http://{agent_llm_addr}/v1/chat/completions"),
            model: "test-model".to_string(),
            tool_servers: Vec::<ToolServerEntry>::new(),
            max_tool_rounds: 4,
            ..AgentConfig::default()
        },
        SessionStore::new()?,
        None,
        native_tools,
    )
    .await?;

    let output = agent
        .run_turn("telegram:bridge", "search available SQL tables")
        .await?;
    assert_eq!(output, "final answer after native wendao search");

    let seen_tool_message = last_tool_message
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default();
    let seen_tool_payload = extract_message_content(&seen_tool_message);
    assert!(
        seen_tool_payload.contains("## Wendao Search"),
        "expected native-tool report in llm second round, got: {seen_tool_payload}"
    );
    assert!(
        seen_tool_payload.contains("sql_table_name=wendao_sql_tables"),
        "expected sql table preview in llm second round, got: {seen_tool_payload}"
    );
    assert_eq!(
        seen_tool_message
            .get("tool_call_id")
            .and_then(serde_json::Value::as_str),
        Some("call_1"),
        "native wendao search tool message should preserve original tool_call_id"
    );

    let observed_queries = gateway_state.observed_queries.lock().await.clone();
    assert_eq!(observed_queries.len(), 3);
    assert_eq!(
        observed_queries[2],
        "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1"
    );

    agent_llm_server.abort();
    let _ = agent_llm_server.await;
    Ok(())
}
