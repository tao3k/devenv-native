//! Block DTOs and block-kind identity helpers.

use std::fmt;
use std::sync::Arc;

/// Shared behavior required for format-local block kind tags.
pub trait BlockKindIdentity {
    /// Returns the stable ID prefix used in generated block IDs.
    fn id_prefix(&self) -> &'static str;
    /// Returns the human-readable display name for this block kind.
    fn display_name(&self) -> &'static str;
}

/// Parser-owned reusable block payload shared across document formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockCore<Kind> {
    /// Stable block identifier scoped to the parent section.
    pub block_id: String,
    /// Format-local block kind variant.
    pub kind: Kind,
    /// Byte range within the parent section content.
    pub byte_range: (usize, usize),
    /// Line range within the parent document.
    pub line_range: (usize, usize),
    /// Compact content fingerprint for block identity.
    pub content_hash: String,
    /// Raw block content including formatting markers.
    pub content: Arc<str>,
    /// Optional explicit property-drawer ID.
    pub id: Option<String>,
    /// Structural heading path for the parent section.
    pub structural_path: Vec<String>,
}

impl<Kind: BlockKindIdentity> BlockCore<Kind> {
    /// Create a new block with an auto-generated block ID.
    #[must_use]
    pub fn new(
        kind: Kind,
        index: usize,
        byte_range: (usize, usize),
        line_range: (usize, usize),
        content: &str,
        structural_path: Vec<String>,
    ) -> Self {
        let block_id = format!("block-{}-{index}", kind.id_prefix());
        let content_hash = compute_block_hash(content);

        Self {
            block_id,
            kind,
            byte_range,
            line_range,
            content_hash,
            content: Arc::from(content),
            id: None,
            structural_path,
        }
    }
}

impl<Kind> BlockCore<Kind> {
    /// Replace the generated block ID with an explicit property-drawer ID.
    #[must_use]
    pub fn with_explicit_id(mut self, id: String) -> Self {
        self.id = Some(id.clone());
        self.block_id = id;
        self
    }
}

/// Markdown-local block kind variants over the shared block contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownBlockKind {
    /// Standard paragraph text.
    Paragraph,
    /// Fenced code block with one optional language tag.
    CodeFence {
        /// Language identifier such as `rust` or `python`.
        language: String,
    },
    /// Ordered or unordered list.
    List {
        /// Whether this is an ordered list.
        ordered: bool,
    },
    /// Blockquote content.
    BlockQuote,
    /// Horizontal rule or thematic break.
    ThematicBreak,
    /// GitHub-flavored Markdown table.
    Table,
    /// Raw HTML block.
    HtmlBlock,
}

impl MarkdownBlockKind {
    /// Returns a short stable identifier for this block kind.
    #[must_use]
    pub fn id_prefix(&self) -> &'static str {
        <Self as BlockKindIdentity>::id_prefix(self)
    }

    /// Returns a human-readable name for this block kind.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        <Self as BlockKindIdentity>::display_name(self)
    }
}

impl BlockKindIdentity for MarkdownBlockKind {
    fn id_prefix(&self) -> &'static str {
        match self {
            Self::Paragraph => "para",
            Self::CodeFence { .. } => "code",
            Self::List { ordered: true } => "olist",
            Self::List { ordered: false } => "ulist",
            Self::BlockQuote => "quote",
            Self::ThematicBreak => "hr",
            Self::Table => "table",
            Self::HtmlBlock => "html",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Paragraph => "Paragraph",
            Self::CodeFence { .. } => "Code Fence",
            Self::List { ordered: true } => "Ordered List",
            Self::List { ordered: false } => "Unordered List",
            Self::BlockQuote => "Block Quote",
            Self::ThematicBreak => "Thematic Break",
            Self::Table => "Table",
            Self::HtmlBlock => "HTML Block",
        }
    }
}

impl fmt::Display for MarkdownBlockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paragraph => write!(f, "Paragraph"),
            Self::CodeFence { language } => write!(f, "CodeFence({language})"),
            Self::List { ordered } => {
                if *ordered {
                    write!(f, "OrderedList")
                } else {
                    write!(f, "UnorderedList")
                }
            }
            Self::BlockQuote => write!(f, "BlockQuote"),
            Self::ThematicBreak => write!(f, "ThematicBreak"),
            Self::Table => write!(f, "Table"),
            Self::HtmlBlock => write!(f, "HtmlBlock"),
        }
    }
}

/// Markdown naming surface over the shared block core.
pub type MarkdownBlock = BlockCore<MarkdownBlockKind>;

impl BlockCore<MarkdownBlockKind> {
    /// Returns the language tag for code fences, if present.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        match &self.kind {
            MarkdownBlockKind::CodeFence { language } => Some(language),
            _ => None,
        }
    }

    /// Returns true when this block is any list variant.
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self.kind, MarkdownBlockKind::List { .. })
    }

    /// Returns true when this block is a fenced code block.
    #[must_use]
    pub fn is_code(&self) -> bool {
        matches!(self.kind, MarkdownBlockKind::CodeFence { .. })
    }
}

/// Compute a compact Blake3 fingerprint for block content.
#[must_use]
pub fn compute_block_hash(content: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();
    hash.to_hex()[..16].to_string()
}
