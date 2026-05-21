//! `ingress::spider::types` owns Wendao ingress spider types behavior.

use std::collections::HashMap;
use std::sync::Arc;

/// Signal emitted after one web page has been assimilated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebIngestionSignal {
    /// Original web URL.
    pub url: String,
    /// Crawl depth used by upstream crawler.
    pub depth: u32,
    /// Stable content hash used for deduplication.
    pub content_hash: String,
}

/// Web page payload accepted by the Spider-to-Wendao bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiderPagePayload {
    /// Absolute source URL.
    pub url: String,
    /// Crawl depth of the source page.
    pub depth: u32,
    /// Optional title extracted by upstream crawler.
    pub title: Option<String>,
    /// Crawled content body (cleaned HTML or markdown-like payload).
    pub markdown_content: Arc<str>,
    /// Additional metadata from upstream crawler/runtime.
    pub metadata: HashMap<String, String>,
}

impl SpiderPagePayload {
    /// Construct payload with required fields.
    #[must_use]
    pub fn new(url: impl Into<String>, depth: u32, markdown_content: impl Into<Arc<str>>) -> Self {
        Self {
            url: url.into(),
            depth,
            title: None,
            markdown_content: markdown_content.into(),
            metadata: HashMap::new(),
        }
    }

    /// Attach optional title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Attach additional metadata map.
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}
