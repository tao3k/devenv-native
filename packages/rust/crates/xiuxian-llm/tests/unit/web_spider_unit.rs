//! Spider bridge unit tests.

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, response::Html};
use tokio::net::TcpListener;
use xiuxian_llm::web::SpiderBridge;

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

#[tokio::test]
async fn spider_bridge_quick_ingest_returns_markdown_context() -> Result<()> {
    let url = spawn_site(
        r#"
<!doctype html>
<html>
  <head>
    <title>Spider Bridge Demo</title>
    <meta name="description" content="Single page compatibility ingest">
  </head>
  <body>
    <h1>Rust Native Bridge</h1>
    <p>First paragraph.</p>
    <ul><li>item one</li><li>item two</li></ul>
    <script>window.secret = "skip-me";</script>
  </body>
</html>
"#,
    )
    .await?;

    let bridge = SpiderBridge::new(url.clone()).with_limit(1);
    let context = bridge.quick_ingest().await?;

    assert_eq!(context.title, "Spider Bridge Demo");
    assert!(
        context.source_url.starts_with(url.as_str()),
        "source url mismatch: {}",
        context.source_url
    );
    assert!(context.markdown_content.contains("Rust Native Bridge"));
    assert!(context.markdown_content.contains("item one"));
    assert!(!context.markdown_content.contains("skip-me"));
    assert_eq!(
        context.metadata.get("engine").map(String::as_str),
        Some("reqwest")
    );
    assert_eq!(
        context.metadata.get("crawler.stealth").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        context
            .metadata
            .get("crawler.content_source")
            .map(String::as_str),
        Some("clean_html")
    );
    assert_eq!(
        context.metadata.get("page.description").map(String::as_str),
        Some("Single page compatibility ingest")
    );
    assert!(context.metadata.contains_key("crawler.user_agent"));

    Ok(())
}

#[tokio::test]
async fn spider_bridge_script_only_page_returns_non_empty_content() -> Result<()> {
    let url = spawn_site(
        r#"
<!doctype html>
<html>
  <body>
    <script>window.only_script_payload = "retain raw fallback";</script>
  </body>
</html>
"#,
    )
    .await?;

    let bridge = SpiderBridge::new(url).with_limit(1);
    let context = bridge.quick_ingest().await?;

    let content_source = context
        .metadata
        .get("crawler.content_source")
        .map(String::as_str);
    assert!(
        matches!(content_source, Some("clean_html" | "raw_html")),
        "unexpected content source: {content_source:?}"
    );
    assert!(
        !context.markdown_content.trim().is_empty(),
        "expected non-empty spider content"
    );

    Ok(())
}
