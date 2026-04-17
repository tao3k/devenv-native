use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::time::{Duration, timeout};
use xiuxian_daochang::test_support::{resolve_runtime_inference_url, resolve_runtime_model};
use xiuxian_daochang::{
    Agent, AgentConfig, NativeToolRegistry, SessionStore, ToolServerEntry, WendaoSearchTool,
    WendaoSearchToolConfig, load_runtime_settings_from_paths,
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

#[derive(Clone, Default)]
struct MockGatewayState {
    observed_queries: Arc<tokio::sync::Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct MockLlmCaptureState {
    requests: Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    round: Arc<AtomicUsize>,
}

fn test_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn live_runtime_default_enabled() -> bool {
    std::env::var("XIUXIAN_DAOCHANG_LIVE_LLM")
        .ok()
        .map(|raw| raw.trim().to_ascii_lowercase())
        .is_some_and(|raw| matches!(raw.as_str(), "1" | "true" | "yes" | "on"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map_or_else(
            || panic!("crate dir should have repo root ancestor"),
            PathBuf::from,
        )
}

fn resolve_live_agent_config() -> Result<Option<AgentConfig>> {
    let repo_root = repo_root();
    let system_settings =
        repo_root.join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user_settings = repo_root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    let runtime_settings = load_runtime_settings_from_paths(&system_settings, &user_settings);
    let inference_url =
        resolve_runtime_inference_url(&runtime_settings, &Vec::<ToolServerEntry>::new())?;
    let model = resolve_runtime_model(&runtime_settings);
    let api_key = runtime_settings
        .inference
        .api_key
        .as_deref()
        .and_then(resolve_configured_api_key);

    if model.trim().is_empty() {
        eprintln!("skip: runtime-default model is unresolved from repo xiuxian.toml");
        return Ok(None);
    }
    if api_key.is_none() {
        eprintln!("skip: runtime-default provider api key is unresolved from repo xiuxian.toml");
        return Ok(None);
    }

    Ok(Some(AgentConfig {
        inference_url,
        model,
        api_key,
        tool_servers: Vec::new(),
        max_tool_rounds: 4,
        ..AgentConfig::default()
    }))
}

fn resolve_configured_api_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(env_name) = trimmed.strip_prefix("env:")
        && is_env_var_name(env_name)
    {
        return std::env::var(env_name)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if trimmed.starts_with("${")
        && trimmed.ends_with('}')
        && trimmed.len() > 3
        && is_env_var_name(&trimmed[2..trimmed.len() - 1])
    {
        return std::env::var(&trimmed[2..trimmed.len() - 1])
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if is_env_var_name(trimmed) {
        return std::env::var(trimmed)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    Some(trimmed.to_string())
}

fn is_env_var_name(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

async fn reserve_local_addr() -> Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = probe.local_addr()?;
    drop(probe);
    Ok(addr)
}

async fn spawn_mock_llm_capture_server(
    addr: std::net::SocketAddr,
) -> Result<(
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
)> {
    let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
    let state = MockLlmCaptureState {
        requests: Arc::clone(&requests),
        round: Arc::new(AtomicUsize::new(0)),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(mock_llm_capture_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        }),
        requests,
    ))
}

async fn mock_llm_capture_handler(
    State(state): State<MockLlmCaptureState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    state
        .requests
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(payload);
    let response = if state.round.fetch_add(1, Ordering::SeqCst) == 0 {
        json!({
            "id": "mock-chatcmpl-capture-1",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": "no tool call needed for capture"
                }
            }]
        })
    } else {
        json!({
            "id": "mock-chatcmpl-capture-2",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": "done"
                }
            }]
        })
    };
    Json(response)
}

async fn spawn_mock_gateway(state: MockGatewayState) -> Result<String> {
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
async fn native_only_agent_advertises_wendao_search_tool_to_llm() -> Result<()> {
    let _guard = test_lock().lock().await;

    let llm_addr = reserve_local_addr().await?;
    let (llm_server, requests) = spawn_mock_llm_capture_server(llm_addr).await?;
    let project_root = tempfile::tempdir()?;
    let gateway_state = MockGatewayState::default();
    let gateway_endpoint = spawn_mock_gateway(gateway_state).await?;

    let mut native_tools = NativeToolRegistry::new();
    native_tools.register(Arc::new(WendaoSearchTool::new_with_llm_mode(
        WendaoSearchToolConfig::new(
            gateway_endpoint,
            Some(project_root.path().display().to_string()),
            HashMap::new(),
        ),
        BootcampLlmMode::Mock {
            response: AUTHOR_RESPONSE_XML.to_string(),
        },
    )));

    let agent = Agent::from_config_with_session_backends_and_native_tools_for_test(
        AgentConfig {
            inference_url: format!("http://{llm_addr}/v1/chat/completions"),
            model: "test-model".to_string(),
            tool_servers: Vec::new(),
            max_tool_rounds: 2,
            ..AgentConfig::default()
        },
        SessionStore::new()?,
        None,
        native_tools,
    )
    .await?;

    let output = agent
        .run_turn(
            "discord:native-tools-capture",
            "Please inspect the available knowledge tool.",
        )
        .await?;
    assert_eq!(output, "no tool call needed for capture");

    let payloads = requests
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let first_payload = payloads
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("mock llm should receive at least one request"));
    let tools = first_payload
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| panic!("native-only agent should send tools to the llm"));
    assert!(
        tools.iter().any(|tool| {
            tool.get("function")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                == Some("wendao.search")
        }),
        "expected wendao.search in first llm request tools payload: {tools:#?}"
    );
    let wendao_search = tools
        .iter()
        .find(|tool| {
            tool.get("function")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                == Some("wendao.search")
        })
        .cloned()
        .unwrap_or_else(|| {
            panic!("expected wendao.search in first llm request tools payload: {tools:#?}")
        });
    assert!(
        wendao_search
            .get("function")
            .and_then(|value| value.get("parameters"))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("query"))
            .is_some(),
        "expected wendao.search schema to expose `query`: {wendao_search:#?}"
    );
    assert_eq!(
        first_payload.get("tool_choice").and_then(Value::as_str),
        Some("auto")
    );

    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn runtime_default_llm_performs_native_wendao_search_tool_call() -> Result<()> {
    if !live_runtime_default_enabled() {
        return Ok(());
    }

    let _guard = test_lock().lock().await;

    let Some(config) = resolve_live_agent_config()? else {
        return Ok(());
    };

    let project_root = tempfile::tempdir()?;
    let gateway_state = MockGatewayState::default();
    let gateway_endpoint = spawn_mock_gateway(gateway_state.clone()).await?;

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
        config,
        SessionStore::new()?,
        None,
        native_tools,
    )
    .await?;

    let output = timeout(
        Duration::from_secs(90),
        agent.run_turn(
            "discord:live-native-tool",
            "You must call the `wendao.search` tool in your very next response. Do not answer from memory. Invoke it exactly once with a request that asks for the available SQL tables. After the tool result arrives, reply with only the first table name and no extra words.",
        ),
    )
    .await
    .map_err(|_| anyhow::anyhow!("live native tool call timed out after 90 seconds"))??;

    let observed_queries = gateway_state.observed_queries.lock().await.clone();
    assert_eq!(
        observed_queries.len(),
        3,
        "expected the live LLM to invoke native wendao.search and reach the mock gateway; output={output}; queries={observed_queries:#?}"
    );
    assert_eq!(
        observed_queries[2],
        "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1"
    );
    assert!(
        output.contains("wendao_sql_tables"),
        "expected final live response to mention first table name, got: {output}"
    );
    Ok(())
}
