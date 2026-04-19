//! Compatibility bridge for single-page web ingestion.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use regex::Regex;
use reqwest::Client;

use crate::llm::error::sanitize_user_visible;
use crate::llm::{LlmError, LlmResult};

const DEFAULT_USER_AGENT: &str = "xiuxian-llm/0.1";
const STEALTH_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) ",
    "AppleWebKit/537.36 (KHTML, like Gecko) ",
    "Chrome/135.0.0.0 Safari/537.36"
);
const REQUEST_TIMEOUT_SECS: u64 = 20;

/// Unified web context returned to runtime callers after one crawl operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebContext {
    /// Source URL that produced this context row.
    pub source_url: String,
    /// Best-effort document title.
    pub title: String,
    /// Best-effort markdown-like normalized body.
    pub markdown_content: Arc<str>,
    /// Transport metadata for telemetry and downstream routing.
    pub metadata: HashMap<String, String>,
}

/// Compatibility wrapper that preserves the Spider bridge surface while using
/// a lighter single-page HTTP fetch implementation.
#[derive(Debug, Clone)]
pub struct SpiderBridge {
    root_url: Arc<str>,
    page_limit: u32,
    stealth_mode: bool,
}

impl SpiderBridge {
    /// Construct one bridge for a root URL.
    #[must_use]
    pub fn new(root_url: impl Into<String>) -> Self {
        Self {
            root_url: Arc::<str>::from(root_url.into()),
            page_limit: 1,
            stealth_mode: true,
        }
    }

    /// Set crawl page limit.
    #[must_use]
    pub fn with_limit(mut self, page_limit: u32) -> Self {
        self.page_limit = page_limit.max(1);
        self
    }

    /// Enable stealth mode.
    #[must_use]
    pub fn with_stealth(mut self, stealth_mode: bool) -> Self {
        self.stealth_mode = stealth_mode;
        self
    }

    /// Execute crawl.
    ///
    /// # Errors
    ///
    /// Returns an error when the page fetch or extraction path cannot produce
    /// usable page content for the configured root URL.
    pub async fn quick_ingest(&self) -> LlmResult<WebContext> {
        // The compatibility bridge keeps the page-limit knob for callers, but
        // this reduced implementation intentionally ingests only the requested
        // source page instead of recursively crawling links.
        let _requested_page_limit = self.page_limit;
        let user_agent = if self.stealth_mode {
            STEALTH_USER_AGENT
        } else {
            DEFAULT_USER_AGENT
        };
        let client = build_client(user_agent)?;
        let response = client
            .get(self.root_url.as_ref())
            .send()
            .await
            .map_err(|error| {
                internal_error(format!("web fetch failed for {}: {error}", self.root_url))
            })?
            .error_for_status()
            .map_err(|error| {
                internal_error(format!(
                    "web fetch returned an error status for {}: {error}",
                    self.root_url
                ))
            })?;

        let source_url = response.url().to_string();
        let html = response.text().await.map_err(|error| {
            internal_error(format!(
                "web fetch response body decode failed for {}: {error}",
                self.root_url
            ))
        })?;
        let cleaned_html = clean_html(html.as_str());

        let (markdown_content, content_source) = resolve_markdown_content(
            cleaned_html.as_str(),
            html.as_str(),
            source_url.as_str(),
            true,
        );

        let title = extract_title(html.as_str()).unwrap_or_else(|| source_url.clone());

        let mut metadata = HashMap::new();
        metadata.insert("engine".to_string(), "reqwest".to_string());
        metadata.insert("crawler.stealth".to_string(), self.stealth_mode.to_string());
        metadata.insert(
            "crawler.content_source".to_string(),
            content_source.to_string(),
        );
        metadata.insert("crawler.user_agent".to_string(), user_agent.to_string());
        if let Some(description) = extract_meta_description(html.as_str()) {
            metadata.insert("page.description".to_string(), description);
        }

        Ok(WebContext {
            source_url,
            title,
            markdown_content,
            metadata,
        })
    }
}

fn build_client(user_agent: &str) -> LlmResult<Client> {
    Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|error| internal_error(format!("failed to build web client: {error}")))
}

fn extract_title(raw_html: &str) -> Option<String> {
    title_regex()
        .captures(raw_html)
        .and_then(|captures| captures.get(1))
        .map(|title| normalize_text(&decode_html_entities(title.as_str())))
        .filter(|title| !title.is_empty())
}

fn extract_meta_description(raw_html: &str) -> Option<String> {
    for meta_tag in meta_tag_regex().find_iter(raw_html) {
        let attributes = parse_html_attributes(meta_tag.as_str());
        let is_description = attributes
            .get("name")
            .or_else(|| attributes.get("property"))
            .is_some_and(|value| {
                value.eq_ignore_ascii_case("description")
                    || value.eq_ignore_ascii_case("og:description")
            });
        if !is_description {
            continue;
        }

        let description = attributes
            .get("content")
            .map(|value| normalize_text(&decode_html_entities(value)))
            .filter(|value| !value.is_empty());
        if description.is_some() {
            return description;
        }
    }

    None
}

fn parse_html_attributes(tag: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    for captures in attribute_regex().captures_iter(tag) {
        let Some(name_match) = captures.get(1) else {
            continue;
        };
        let Some(value_match) = captures
            .get(2)
            .or_else(|| captures.get(3))
            .or_else(|| captures.get(4))
        else {
            continue;
        };

        attributes.insert(
            name_match.as_str().to_ascii_lowercase(),
            value_match.as_str().to_string(),
        );
    }

    attributes
}

fn clean_html(raw_html: &str) -> String {
    let without_comments = html_comment_regex().replace_all(raw_html, " ");
    let without_noise = noise_block_regex().replace_all(&without_comments, " ");
    let with_block_breaks = block_break_regex().replace_all(&without_noise, "\n");
    let without_tags = html_tag_regex().replace_all(&with_block_breaks, " ");
    normalize_text(&decode_html_entities(without_tags.as_ref()))
}

fn normalize_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_html_entities(input: &str) -> String {
    let mut decoded = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(entity_start) = remaining.find('&') {
        decoded.push_str(&remaining[..entity_start]);
        let entity_tail = &remaining[entity_start + 1..];
        let Some(entity_end) = entity_tail.find(';') else {
            decoded.push('&');
            remaining = entity_tail;
            continue;
        };

        let entity_name = &entity_tail[..entity_end];
        if let Some(value) = decode_html_entity(entity_name) {
            decoded.push(value);
            remaining = &entity_tail[entity_end + 1..];
            continue;
        }

        decoded.push('&');
        remaining = entity_tail;
    }

    decoded.push_str(remaining);
    decoded
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn title_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?is)<title\b[^>]*>(.*?)</title>").expect("valid title regex"))
}

fn meta_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<meta\b[^>]*>").expect("valid meta tag regex"))
}

fn attribute_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?is)([A-Za-z_:][A-Za-z0-9_:\-\.]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))"#,
        )
        .expect("valid html attribute regex")
    })
}

fn html_comment_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<!--.*?-->").expect("valid html comment regex"))
}

fn noise_block_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)<(?:head|script|style|noscript|template|svg|math)\b.*?</(?:head|script|style|noscript|template|svg|math)>")
            .expect("valid noisy block regex")
    })
}

fn block_break_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?is)</?(?:article|aside|blockquote|br|dd|div|dl|dt|fieldset|figcaption|figure|footer|form|h[1-6]|header|hr|li|main|nav|ol|p|pre|section|table|tbody|td|tfoot|th|thead|tr|ul)\b[^>]*>",
        )
        .expect("valid block break regex")
    })
}

fn html_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid html tag regex"))
}

pub(super) fn resolve_markdown_content(
    cleaned_text: &str,
    raw_html: &str,
    url: &str,
    prefer_raw: bool,
) -> (Arc<str>, &'static str) {
    if !cleaned_text.trim().is_empty() {
        return (Arc::from(cleaned_text), "clean_html");
    }
    if !raw_html.trim().is_empty() && prefer_raw {
        return (Arc::from(raw_html), "raw_html");
    }
    (Arc::from(url), "url_fallback")
}

fn internal_error(message: impl Into<String>) -> LlmError {
    LlmError::Internal {
        message: sanitize_user_visible(message.into().as_str()),
    }
}
