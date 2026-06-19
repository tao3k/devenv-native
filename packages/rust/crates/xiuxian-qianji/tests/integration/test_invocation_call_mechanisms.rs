//! Integration tests for contract-validated HTTP and CLI invocation nodes.

#![cfg(feature = "wendao-integration")]

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

fn build_compiler(index_root: &Path) -> Result<QianjiCompiler, Box<dyn std::error::Error>> {
    let index = Arc::new(LinkGraphIndex::build(index_root)?);
    Ok(QianjiCompiler::new(index))
}

#[tokio::test]
async fn http_call_node_executes_with_contract_validated_request()
-> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        let app = Router::new().route("/api/docs/navigation", get(mock_navigation));
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|error| panic!("mock navigation server should serve: {error}"));
    });

    let manifest = format!(
        r#"
name = "HttpInvocation"

[[nodes]]
id = "OpenNavigation"
kind = "http_call"
contract = "wendao.docs.navigation"
method = "GET"
path = "http://{address}/api/docs/navigation"
query = {{ repo = "$repo", page_id = "$page_id", related_limit = 5, family_limit = 3 }}
"#
    );
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(&manifest)?;
    let scheduler = QianjiScheduler::new(engine);
    let output = scheduler
        .run(json!({
            "repo": "demo",
            "page_id": "intro"
        }))
        .await?;

    assert_eq!(output["OpenNavigation"]["transport"], "http");
    assert_eq!(output["OpenNavigation"]["status"], 200);
    assert_eq!(output["OpenNavigation"]["body"]["query"]["repo"], "demo");
    assert_eq!(
        output["OpenNavigation"]["body"]["query"]["page_id"],
        "intro"
    );
    assert_eq!(
        output["OpenNavigation"]["body"]["query"]["related_limit"],
        "5"
    );
    assert_eq!(
        output["OpenNavigation"]["body"]["query"]["family_limit"],
        "3"
    );
    Ok(())
}

#[tokio::test]
async fn cli_call_node_executes_with_contract_validated_argv()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let cli_path = temp.path().join("wendao");
    std::fs::write(&cli_path, "#!/bin/sh\nprintf '%s|' \"$@\"\n")?;
    let mut permissions = std::fs::metadata(&cli_path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        std::fs::set_permissions(&cli_path, permissions)?;
    }

    let manifest = format!(
        r#"
name = "CliInvocation"

[[nodes]]
id = "OpenNavigationCli"
kind = "cli_call"
contract = "wendao.docs.navigation"
argv = ["{}", "docs", "navigation", "--repo", "$repo", "--page-id", "$page_id", "--related-limit", "5", "--family-limit", "3"]
"#,
        cli_path.display()
    );
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(&manifest)?;
    let scheduler = QianjiScheduler::new(engine);
    let output = scheduler
        .run(json!({
            "repo": "demo",
            "page_id": "intro"
        }))
        .await?;

    let stdout = output["OpenNavigationCli"]["stdout"]
        .as_str()
        .unwrap_or_else(|| panic!("cli_call should expose stdout"));
    assert_eq!(output["OpenNavigationCli"]["transport"], "cli");
    assert!(stdout.contains("docs|navigation|--repo|demo|"));
    assert!(stdout.contains("--page-id|intro|"));
    Ok(())
}

async fn mock_navigation(Query(query): Query<BTreeMap<String, String>>) -> Json<Value> {
    Json(json!({
        "query": query
    }))
}
