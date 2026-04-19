use std::sync::Arc;

use serde_json::{Value, json};
use xiuxian_llm::web::{SpiderBridge, WebContext};
use xiuxian_wendao::ingress::{SpiderPagePayload, SpiderWendaoBridge, canonical_web_uri};

use super::macros::define_native_tool;

const DEFAULT_PAGE_LIMIT: u32 = 1;
const MAX_PAGE_LIMIT: u32 = 8;
const DEFAULT_PREVIEW_CHAR_LIMIT: usize = 1200;

define_native_tool! {
    /// Native tool for crawling one web page via the compatibility web bridge and optionally persisting into Wendao.
    pub struct SpiderCrawlTool {
        /// Optional Wendao ingress bridge. When present, crawled content is assimilated into graph.
        pub ingress: Option<Arc<SpiderWendaoBridge>>,
    }
    name: "web.crawl",
    description: "Fetch one web page and return a cleaned markdown preview. Use when user asks to fetch, summarize, or extract content from a URL. Optionally persists into the Wendao graph.",
    parameters: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Absolute URL to crawl (http or https)."
                },
                "page_limit": {
                    "type": "integer",
                    "description": "Requested page limit for compatibility callers (1-8). Default: 1.",
                    "minimum": 1,
                    "maximum": 8
                },
                "stealth": {
                    "type": "boolean",
                    "description": "Enable a browser-like user agent hint. Default: true."
                },
                "persist_to_wendao": {
                    "type": "boolean",
                    "description": "Persist crawled content into Wendao graph when ingress is available. Default: true."
                },
                "include_preview": {
                    "type": "boolean",
                    "description": "Include cleaned text preview in output. Default: true."
                }
            },
            "required": ["url"]
        }),
    call(|tool, arguments, _context| {
        let url = require_url(arguments.as_ref())?;
        let page_limit = parse_page_limit(arguments.as_ref());
        let stealth = parse_bool(arguments.as_ref(), "stealth", true);
        let persist_to_wendao = parse_bool(arguments.as_ref(), "persist_to_wendao", true);
        let include_preview = parse_bool(arguments.as_ref(), "include_preview", true);

        let context = SpiderBridge::new(url)
            .with_limit(page_limit)
            .with_stealth(stealth)
            .quick_ingest()
            .await?;

        let mut lines = vec![
            "## Web Crawl Result".to_string(),
            format!("- Source: {}", context.source_url),
            format!("- Title: {}", non_empty_or_fallback(context.title.as_str(), context.source_url.as_str())),
            format!("- Engine: {}", context.metadata.get("engine").map_or("spider", String::as_str)),
        ];

        if let Ok(uri) = canonical_web_uri(context.source_url.as_str()) {
            lines.push(format!("- Wendao URI: `{uri}`"));
        }

        let persistence_status = maybe_persist_to_wendao(tool.ingress.as_ref(), &context, persist_to_wendao);
        lines.push(format!("- Wendao Persistence: {persistence_status}"));

        if include_preview {
            lines.push(String::new());
            lines.push("### Preview".to_string());
            lines.push(preview_text(context.markdown_content.as_ref(), DEFAULT_PREVIEW_CHAR_LIMIT));
        }

        Ok(lines.join("\n"))
    })
}

fn require_url(arguments: Option<&Value>) -> anyhow::Result<String> {
    let url = arguments
        .and_then(|args| args.get("url"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing 'url' argument"))?;
    Ok(url.to_string())
}

fn parse_page_limit(arguments: Option<&Value>) -> u32 {
    let raw = arguments
        .and_then(|args| args.get("page_limit"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(DEFAULT_PAGE_LIMIT);
    raw.clamp(1, MAX_PAGE_LIMIT)
}

fn parse_bool(arguments: Option<&Value>, key: &str, default: bool) -> bool {
    arguments
        .and_then(|args| args.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(default)
}

fn maybe_persist_to_wendao(
    ingress: Option<&Arc<SpiderWendaoBridge>>,
    context: &WebContext,
    persist_to_wendao: bool,
) -> String {
    if !persist_to_wendao {
        return "skipped (persist_to_wendao=false)".to_string();
    }

    let Some(ingress) = ingress else {
        return "unavailable (no Wendao graph ingress mounted)".to_string();
    };

    let mut payload = SpiderPagePayload::new(
        context.source_url.clone(),
        0,
        Arc::clone(&context.markdown_content),
    )
    .with_metadata(context.metadata.clone());

    if !context.title.trim().is_empty() && context.title != context.source_url {
        payload = payload.with_title(context.title.clone());
    }

    match ingress.ingest_page(&payload) {
        Ok(Some(signal)) => format!("ingested (hash={})", signal.content_hash),
        Ok(None) => "deduplicated (existing content hash)".to_string(),
        Err(error) => format!("failed ({error})"),
    }
}

fn preview_text(content: &str, limit: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.is_empty() {
        return "(empty content)".to_string();
    }

    let mut preview = normalized.chars().take(limit).collect::<String>();
    if normalized.chars().count() > limit {
        preview.push_str(" ...");
    }
    preview
}

fn non_empty_or_fallback<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}
