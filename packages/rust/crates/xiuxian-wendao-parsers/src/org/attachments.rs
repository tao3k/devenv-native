//! Parser-owned Org attachment link extraction.

use orgize::Org;
use orgize::rowan::ast::AstNode;
use orgize::syntax_ast::SyntaxLink;
use serde::{Deserialize, Serialize};

/// Parser-owned Org attachment link record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgAttachmentLink {
    /// Original Org link path, including the protocol prefix and any search suffix.
    pub raw_path: String,
    /// Resolved target path with the protocol prefix and Org search suffix removed.
    pub target_path: String,
    /// Link protocol used by the Org source.
    pub protocol: OrgAttachmentLinkProtocol,
    /// Raw link description text, or an empty string when the link has no description.
    pub description: String,
    /// Optional caption text attached to the link.
    pub caption: Option<String>,
    /// One-based line number where the link starts in the source document.
    pub line: usize,
}

/// Supported Org attachment link protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgAttachmentLinkProtocol {
    /// Standard Org file link, for example `file:relative/path.pdf`.
    File,
    /// Org attachment link, for example `attachment:evidence.pdf`.
    Attachment,
}

/// Extract attachment-bearing Org links from source content.
#[must_use]
pub fn extract_org_attachment_links(content: &str) -> Vec<OrgAttachmentLink> {
    let org = Org::parse(content);
    org.syntax_document()
        .syntax()
        .descendants()
        .filter_map(SyntaxLink::cast)
        .filter_map(|link| org_attachment_link_from_syntax(content, &link))
        .collect()
}

fn org_attachment_link_from_syntax(content: &str, link: &SyntaxLink) -> Option<OrgAttachmentLink> {
    let raw_path = link.path().to_string();
    let (protocol, target_path) = parse_org_attachment_path(raw_path.as_str())?;
    let description = link.description_raw().trim().to_string();
    let caption = link
        .caption()
        .and_then(|caption| caption.value().map(|value| value.trim().to_string()))
        .filter(|value| !value.is_empty());
    let start = u32::from(link.start()) as usize;

    Some(OrgAttachmentLink {
        raw_path,
        target_path,
        protocol,
        description,
        caption,
        line: line_number_at_byte_offset(content, start),
    })
}

fn parse_org_attachment_path(raw_path: &str) -> Option<(OrgAttachmentLinkProtocol, String)> {
    let (raw_protocol, raw_target) = raw_path.split_once(':')?;
    let protocol = match raw_protocol.trim().to_ascii_lowercase().as_str() {
        "file" => OrgAttachmentLinkProtocol::File,
        "attachment" => OrgAttachmentLinkProtocol::Attachment,
        _ => return None,
    };
    let target_path = raw_target
        .split_once("::")
        .map_or(raw_target, |(target, _search)| target)
        .trim()
        .to_string();
    (!target_path.is_empty()).then_some((protocol, target_path))
}

fn line_number_at_byte_offset(content: &str, offset: usize) -> usize {
    content
        .as_bytes()
        .iter()
        .take(offset.min(content.len()))
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}
