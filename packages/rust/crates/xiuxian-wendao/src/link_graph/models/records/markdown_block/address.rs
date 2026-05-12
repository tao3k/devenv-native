//! `link_graph::models::records::markdown_block::address` owns Wendao records markdown block address behavior.

use std::fmt;

/// Block kind specifier for path-based addressing.
///
/// Used in block address paths like `/Section/Paragraph[2]` or `/Section/CodeFence[0]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockKindSpecifier {
    /// Match paragraph blocks.
    Paragraph,
    /// Match code fence blocks.
    CodeFence,
    /// Match list blocks (ordered or unordered).
    List,
    /// Match blockquote blocks.
    BlockQuote,
    /// Match list items within a list.
    Item,
}

impl BlockKindSpecifier {
    /// Parse a block kind specifier from a string.
    ///
    /// # Supported formats
    ///
    /// - `Paragraph` or `para` - Paragraph blocks
    /// - `CodeFence`, `Code`, or `code` - Code fence blocks
    /// - `List` or `list` - List blocks
    /// - `BlockQuote`, `Quote`, or `quote` - Blockquote blocks
    /// - `Item` or `item` - List items
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "paragraph" | "para" => Some(Self::Paragraph),
            "codefence" | "code" => Some(Self::CodeFence),
            "list" => Some(Self::List),
            "blockquote" | "quote" => Some(Self::BlockQuote),
            "item" => Some(Self::Item),
            _ => None,
        }
    }

    /// Get the canonical name for this specifier.
    #[must_use]
    pub fn canonical_name(&self) -> &'static str {
        match self {
            Self::Paragraph => "Paragraph",
            Self::CodeFence => "CodeFence",
            Self::List => "List",
            Self::BlockQuote => "BlockQuote",
            Self::Item => "Item",
        }
    }
}

impl fmt::Display for BlockKindSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

/// Address for a specific block within a section.
///
/// Used in conjunction with a section path to address blocks like:
/// `/Section/Paragraph[2]` or `/Section/List[1]/Item[3]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockAddress {
    /// The block kind to match.
    pub kind: BlockKindSpecifier,
    /// The index of the matching block (0-based).
    pub index: usize,
    /// For list items: the item index within the parent list.
    pub sub_index: Option<usize>,
}

impl BlockAddress {
    /// Create a new block address.
    #[must_use]
    pub fn new(kind: BlockKindSpecifier, index: usize) -> Self {
        Self {
            kind,
            index,
            sub_index: None,
        }
    }

    /// Create a block address for a list item.
    #[must_use]
    pub fn list_item(list_index: usize, item_index: usize) -> Self {
        Self {
            kind: BlockKindSpecifier::List,
            index: list_index,
            sub_index: Some(item_index),
        }
    }

    /// Parse a block address from a path component.
    ///
    /// # Formats
    ///
    /// - `Paragraph[2]` - Second paragraph in section
    /// - `CodeFence[0]` - First code fence
    /// - `List[1]/Item[3]` - Third item in second list
    ///
    /// Returns `(kind, index, sub_index)` tuple.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let open_bracket = s.find('[')?;
        let close_bracket = s.find(']')?;

        if close_bracket <= open_bracket {
            return None;
        }

        let kind_str = &s[..open_bracket];
        let index_str = &s[open_bracket + 1..close_bracket];

        let kind = BlockKindSpecifier::parse(kind_str)?;
        let index = index_str.parse().ok()?;

        let rest = &s[close_bracket + 1..];
        let sub_index = if rest.starts_with("/Item[") {
            let item_open = rest.find('[')?;
            let item_close = rest.find(']')?;
            if item_close > item_open {
                rest[item_open + 1..item_close].parse().ok()
            } else {
                None
            }
        } else {
            None
        };

        Some(Self {
            kind,
            index,
            sub_index,
        })
    }

    /// Format as a path component string.
    #[must_use]
    pub fn to_path_component(&self) -> String {
        match self.sub_index {
            Some(item_idx) => format!(
                "{}[{}]/Item[{}]",
                self.kind.canonical_name(),
                self.index,
                item_idx
            ),
            None => format!("{}[{}]", self.kind.canonical_name(), self.index),
        }
    }
}
