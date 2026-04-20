//! Integration tests for bounded Wendao SQL authoring mechanisms.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::net::TcpListener;
use xiuxian_qianji::executors::{
    WendaoSqlDiscoverMechanism, WendaoSqlExecuteMechanism, WendaoSqlValidateMechanism,
};
use xiuxian_qianji::{FlowInstruction, QianjiMechanism};

xiuxian_testing::crate_test_policy_harness!();

#[derive(Clone, Default)]
struct MockGatewayState {
    observed_queries: Arc<tokio::sync::Mutex<Vec<String>>>,
}

#[tokio::test]
async fn wendao_sql_mechanisms_complete_bounded_success_path()
-> Result<(), Box<dyn std::error::Error>> {
    let state = MockGatewayState::default();
    let endpoint = spawn_mock_gateway(state.clone()).await?;

    let discover = WendaoSqlDiscoverMechanism {
        output_key: "surface_bundle_xml".to_string(),
        endpoint_key: None,
        endpoint: Some(endpoint.clone()),
        project_root_key: Some("project_root".to_string()),
        allowed_objects: vec!["wendao_sql_tables".to_string()],
        max_limit: 8,
        allowed_ops: vec!["eq".to_string(), "contains".to_string()],
        require_filter_for: Vec::new(),
    };
    let validate = WendaoSqlValidateMechanism {
        surface_bundle_key: "surface_bundle_xml".to_string(),
        author_spec_key: "author_spec_xml".to_string(),
        output_key: "validated_sql".to_string(),
        report_key: "validation_report_xml".to_string(),
        error_key: "validation_error".to_string(),
        accepted_branch_label: Some("validate.ok".to_string()),
        rejected_branch_label: Some("validate.repair".to_string()),
    };
    let execute = WendaoSqlExecuteMechanism {
        sql_key: "validated_sql".to_string(),
        output_key: "sql_query_payload".to_string(),
        report_key: "execution_report_xml".to_string(),
        error_key: "execution_error".to_string(),
        endpoint_key: None,
        endpoint: Some(endpoint),
        success_branch_label: Some("execute.done".to_string()),
        error_branch_label: Some("execute.failed".to_string()),
        max_report_rows: 2,
    };

    let mut context = json!({
        "project_root": "/tmp/project",
        "author_spec_xml": r"
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
        "
    });

    let discover_output = discover.execute(&context).await?;
    assert!(matches!(
        discover_output.instruction,
        FlowInstruction::Continue
    ));
    merge_context(&mut context, &discover_output.data);

    let validate_output = validate.execute(&context).await?;
    assert!(matches!(
        validate_output.instruction,
        FlowInstruction::SelectBranch(ref label) if label == "validate.ok"
    ));
    assert_eq!(
        validate_output.data["validated_sql"].as_str(),
        Some("SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1")
    );
    merge_context(&mut context, &validate_output.data);

    let execute_output = execute.execute(&context).await?;
    assert!(matches!(
        execute_output.instruction,
        FlowInstruction::SelectBranch(ref label) if label == "execute.done"
    ));
    assert_eq!(
        execute_output.data["sql_query_payload"]["metadata"]["resultRowCount"].as_u64(),
        Some(1)
    );
    assert!(
        execute_output.data["execution_report_xml"]
            .as_str()
            .is_some_and(|xml| xml.contains("<status>success</status>"))
    );

    let observed_queries = state.observed_queries.lock().await.clone();
    assert_eq!(observed_queries.len(), 3);
    assert_eq!(
        observed_queries[2],
        "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name ASC LIMIT 1"
    );

    Ok(())
}

#[tokio::test]
async fn wendao_sql_validate_routes_invalid_spec_to_repair_branch()
-> Result<(), Box<dyn std::error::Error>> {
    let mechanism = WendaoSqlValidateMechanism {
        surface_bundle_key: "surface_bundle_xml".to_string(),
        author_spec_key: "author_spec_xml".to_string(),
        output_key: "validated_sql".to_string(),
        report_key: "validation_report_xml".to_string(),
        error_key: "validation_error".to_string(),
        accepted_branch_label: Some("validate.ok".to_string()),
        rejected_branch_label: Some("validate.repair".to_string()),
    };

    let output = mechanism
        .execute(&json!({
            "surface_bundle_xml": r"
            <surface_bundle>
              <project_root>/tmp/project</project_root>
              <catalog_table_name>wendao_sql_tables</catalog_table_name>
              <column_catalog_table_name>wendao_sql_columns</column_catalog_table_name>
              <view_source_catalog_table_name>wendao_sql_view_sources</view_source_catalog_table_name>
              <policy>
                <max_limit>8</max_limit>
                <allowed_op>eq</allowed_op>
              </policy>
              <objects>
                <object>
                  <name>wendao_sql_tables</name>
                  <kind>table</kind>
                  <scope>request</scope>
                  <corpus>catalog</corpus>
                  <source_count>1</source_count>
                  <columns>
                    <column>
                      <name>sql_table_name</name>
                      <data_type>Utf8</data_type>
                      <nullable>false</nullable>
                      <ordinal_position>1</ordinal_position>
                      <origin_kind>physical</origin_kind>
                    </column>
                  </columns>
                </object>
              </objects>
            </surface_bundle>
            ",
            "author_spec_xml": r"
            <sql_author_spec>
              <target_object>wendao_sql_tables</target_object>
              <projection>
                <column>missing_column</column>
              </projection>
              <limit>1</limit>
            </sql_author_spec>
            "
        }))
        .await?;

    assert!(matches!(
        output.instruction,
        FlowInstruction::SelectBranch(ref label) if label == "validate.repair"
    ));
    assert!(
        output.data["validation_error"]
            .as_str()
            .is_some_and(|message| message.contains("missing_column"))
    );

    Ok(())
}

fn merge_context(context: &mut Value, patch: &Value) {
    let Some(context_object) = context.as_object_mut() else {
        panic!("context should remain an object");
    };
    let Some(patch_object) = patch.as_object() else {
        panic!("patch should be an object");
    };
    for (key, value) in patch_object {
        context_object.insert(key.clone(), value.clone());
    }
}

async fn spawn_mock_gateway(state: MockGatewayState) -> Result<String, Box<dyn std::error::Error>> {
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
                "registeredTables": ["wendao_sql_tables", "wendao_sql_columns", "wendao_sql_view_sources"],
                "registeredTableCount": 3,
                "registeredViewCount": 0,
                "registeredColumnCount": 6,
                "registeredViewSourceCount": 0,
                "resultBatchCount": 1,
                "resultRowCount": rows.len()
            },
            "batches": [{
                "rowCount": rows.len(),
                "columns": columns,
                "rows": rows
            }]
        }
    })
}
