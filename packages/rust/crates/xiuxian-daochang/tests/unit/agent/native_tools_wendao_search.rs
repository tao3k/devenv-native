use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::net::TcpListener;
use xiuxian_daochang::{
    NativeTool, NativeToolCallContext, WendaoSearchTool, WendaoSearchToolConfig,
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

#[tokio::test]
async fn wendao_search_tool_runs_bounded_workflow_against_gateway() -> Result<()> {
    let state = MockGatewayState::default();
    let endpoint = spawn_mock_gateway(state.clone()).await?;
    let default_root = tempdir()?;
    let session_root = tempdir()?;
    let tool = WendaoSearchTool::new_with_llm_mode(
        WendaoSearchToolConfig::new(
            endpoint.clone(),
            Some(default_root.path().display().to_string()),
            HashMap::from([(
                "discord:123".to_string(),
                session_root.path().display().to_string(),
            )]),
        ),
        BootcampLlmMode::Mock {
            response: AUTHOR_RESPONSE_XML.to_string(),
        },
    );

    let output = tool
        .call(
            Some(json!({
                "query": "Show me the available SQL tables."
            })),
            &NativeToolCallContext {
                session_id: Some("discord:123".to_string()),
                tool_call_id: Some("call_123".to_string()),
            },
        )
        .await?;

    assert!(output.contains("## Wendao Search"));
    assert!(output.contains("- Status: success"));
    assert!(output.contains("Project Root"));
    assert!(output.contains(session_root.path().display().to_string().as_str()));
    assert!(output.contains("SELECT sql_table_name FROM wendao_sql_tables"));
    assert!(output.contains("sql_table_name=wendao_sql_tables"));

    let observed_queries = state.observed_queries.lock().await.clone();
    assert_eq!(observed_queries.len(), 3);
    assert_eq!(
        observed_queries[2],
        "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1"
    );
    Ok(())
}

#[tokio::test]
async fn wendao_search_tool_accepts_legacy_request_alias() -> Result<()> {
    let state = MockGatewayState::default();
    let endpoint = spawn_mock_gateway(state.clone()).await?;
    let project_root = tempdir()?;
    let tool = WendaoSearchTool::new_with_llm_mode(
        WendaoSearchToolConfig::new(
            endpoint,
            Some(project_root.path().display().to_string()),
            HashMap::new(),
        ),
        BootcampLlmMode::Mock {
            response: AUTHOR_RESPONSE_XML.to_string(),
        },
    );

    let output = tool
        .call(
            Some(json!({
                "request": "Show me the available SQL tables."
            })),
            &NativeToolCallContext::default(),
        )
        .await?;

    assert!(output.contains("- Status: success"));
    let observed_queries = state.observed_queries.lock().await.clone();
    assert_eq!(observed_queries.len(), 3);
    Ok(())
}

async fn spawn_mock_gateway(state: MockGatewayState) -> Result<String> {
    let app = Router::new()
        .route("/query", post(mock_query_handler))
        .with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
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
