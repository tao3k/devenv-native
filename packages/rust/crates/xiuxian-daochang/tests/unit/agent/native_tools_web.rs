use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use serde_json::json;
use tempfile::tempdir;
use tokio::net::TcpListener;
use xiuxian_daochang::{NativeTool, NativeToolCallContext, SpiderCrawlTool};
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::ingress::{SpiderWendaoBridge, canonical_web_uri};
use xiuxian_zhixing::{ZhixingHeyi, storage::MarkdownStorage};

#[derive(Clone)]
struct PageState {
    html: &'static str,
}

async fn page(State(state): State<PageState>) -> impl IntoResponse {
    (StatusCode::OK, Html(state.html))
}

async fn spawn_site(html: &'static str) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/", get(page))
        .with_state(PageState { html });
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(format!("http://{addr}/"))
}

fn build_heyi() -> Result<(Arc<ZhixingHeyi>, tempfile::TempDir)> {
    let graph = Arc::new(KnowledgeGraph::new());
    let tmp = tempdir()?;
    let storage = Arc::new(MarkdownStorage::new(tmp.path().to_path_buf()));
    let manifestation = Arc::new(ManifestationManager::new_with_embedded_templates(
        &[],
        &[(
            "task_add_response.md",
            "Mock Manifestation Content -> {{ task_title }}",
        )],
    )?);
    let heyi = ZhixingHeyi::new(
        graph,
        manifestation,
        storage,
        "web-native-tools-test".to_string(),
        "UTC",
    )?;
    Ok((Arc::new(heyi), tmp))
}

#[tokio::test]
async fn spider_crawl_tool_returns_preview_without_ingress() -> Result<()> {
    let url = spawn_site(
        r"
<!doctype html>
<html>
  <head><title>Spider Tool Page</title></head>
  <body>
    <h1>Knowledge from Web</h1>
    <p>Native spider tool can fetch this page.</p>
  </body>
</html>
",
    )
    .await?;

    let tool = SpiderCrawlTool { ingress: None };
    let output = tool
        .call(
            Some(json!({
                "url": url.clone(),
                "include_preview": true,
                "persist_to_wendao": true,
            })),
            &NativeToolCallContext::default(),
        )
        .await?;

    assert!(output.contains("Web Crawl Result"));
    assert!(output.contains("Spider Tool Page"));
    assert!(output.contains("Knowledge from Web"));
    assert!(output.contains("unavailable (no Wendao graph ingress mounted)"));
    Ok(())
}

#[tokio::test]
async fn spider_crawl_tool_ingests_into_wendao_and_deduplicates() -> Result<()> {
    let (heyi, _tmp) = build_heyi()?;
    let ingress = Arc::new(SpiderWendaoBridge::for_shared_knowledge_graph(Arc::clone(
        &heyi.graph,
    )));

    let url = spawn_site(
        r"
<!doctype html>
<html>
  <head><title>Ingress Page</title></head>
  <body>
    <p>Persist me into Wendao graph.</p>
  </body>
</html>
",
    )
    .await?;

    let tool = SpiderCrawlTool {
        ingress: Some(Arc::clone(&ingress)),
    };

    let first = tool
        .call(
            Some(json!({
                "url": url.clone(),
                "include_preview": false,
                "persist_to_wendao": true,
            })),
            &NativeToolCallContext::default(),
        )
        .await?;
    assert!(first.contains("ingested (hash="));

    let second = tool
        .call(
            Some(json!({
                "url": url.clone(),
                "include_preview": false,
                "persist_to_wendao": true,
            })),
            &NativeToolCallContext::default(),
        )
        .await?;
    assert!(second.contains("deduplicated (existing content hash)"));

    let uri = canonical_web_uri(url.as_str())?;
    let entity = heyi.graph.get_entity_by_name(uri.as_str());
    assert!(entity.is_some(), "ingested web document should be in graph");

    Ok(())
}
