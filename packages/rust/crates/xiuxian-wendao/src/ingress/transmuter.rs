//! Lightweight content washing for Spider ingress.

use thiserror::Error;

/// Structural validation failures detected during ingress washing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(super) enum IngressTransmuterError {
    /// Input contains null bytes and is rejected before assimilation.
    #[error("input contains null bytes")]
    NullByteDetected,
    /// Closing tag did not match the latest opening tag.
    #[error("mismatched XML-Lite tag: expected </{expected}>, found </{found}>")]
    MismatchedClosingTag {
        /// The opening tag waiting to be closed.
        expected: String,
        /// The closing tag found in the payload.
        found: String,
    },
    /// Closing tag appeared without a corresponding opening tag.
    #[error("unexpected XML-Lite closing tag </{found}>")]
    UnexpectedClosingTag {
        /// The closing tag that could not be matched.
        found: String,
    },
    /// Input ended while some opening tags were still unclosed.
    #[error("unclosed XML-Lite tag <{tag}>")]
    UnclosedTag {
        /// The opening tag that remained on stack.
        tag: String,
    },
}

/// Failures for content washing plus structural validation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(super) enum ResolveAndWashError {
    /// The supplied content was empty after trimming.
    #[error("semantic resource URI `{uri}` could not be resolved")]
    ResourceNotFound {
        /// Canonical resource URI.
        uri: String,
    },
    /// Structural validation failed after content washing.
    #[error(transparent)]
    Transmuter(#[from] IngressTransmuterError),
}

/// Resolve one already-loaded payload and apply lightweight washing.
///
/// # Errors
///
/// Returns [`ResolveAndWashError::ResourceNotFound`] when `raw_content` is blank.
/// Returns [`ResolveAndWashError::Transmuter`] when XML-Lite validation fails.
pub(super) fn resolve_and_wash(
    uri: &str,
    raw_content: &str,
) -> Result<String, ResolveAndWashError> {
    let canonical_uri = uri.trim();
    if raw_content.trim().is_empty() {
        return Err(ResolveAndWashError::ResourceNotFound {
            uri: canonical_uri.to_string(),
        });
    }

    let raw = raw_content;
    let refined = refine_for_llm(raw);
    if should_validate_xml_lite(canonical_uri) {
        validate_structure(refined.as_str())?;
    }
    Ok(refined)
}

fn refine_for_llm(content: &str) -> String {
    let normalized_line_endings = content.replace("\r\n", "\n").replace('\r', "\n");
    let sanitized = normalized_line_endings.replace('\0', "");

    let mut refined = String::with_capacity(sanitized.len());
    let mut blank_run = 0usize;
    for line in sanitized.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            blank_run += 1;
            if blank_run > 2 {
                continue;
            }
        } else {
            blank_run = 0;
        }

        if !refined.is_empty() {
            refined.push('\n');
        }
        refined.push_str(trimmed_end);
    }

    refined.trim().to_string()
}

fn validate_structure(content: &str) -> Result<(), IngressTransmuterError> {
    if content.contains('\0') {
        return Err(IngressTransmuterError::NullByteDetected);
    }

    let mut cursor = 0usize;
    let mut stack: Vec<String> = Vec::new();

    while let Some(tag) = scan_xml_lite_tag(content, cursor)? {
        cursor = tag.next_cursor;
        apply_xml_lite_tag(&mut stack, tag)?;
    }

    if let Some(tag) = stack.pop() {
        return Err(IngressTransmuterError::UnclosedTag { tag });
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct XmlLiteTag<'a> {
    name: &'a str,
    closing: bool,
    self_closing: bool,
    next_cursor: usize,
}

fn scan_xml_lite_tag(
    content: &str,
    cursor: usize,
) -> Result<Option<XmlLiteTag<'_>>, IngressTransmuterError> {
    let Some(open_offset) = content[cursor..].find('<') else {
        return Ok(None);
    };
    let open = cursor + open_offset;
    let bytes = content.as_bytes();
    if open + 1 >= bytes.len() {
        return Ok(None);
    }

    match bytes[open + 1] {
        b'!' => scan_xml_lite_decl_or_comment(content, open),
        b'?' => scan_xml_lite_processing_instruction(content, open),
        b'/' => scan_named_xml_lite_tag(content, open, true),
        _ => scan_named_xml_lite_tag(content, open, false),
    }
}

fn scan_xml_lite_decl_or_comment(
    content: &str,
    open: usize,
) -> Result<Option<XmlLiteTag<'_>>, IngressTransmuterError> {
    if !content[open..].starts_with("<!--") {
        return scan_xml_lite_tag(content, open + 1);
    }
    let Some(offset) = content[open + 4..].find("-->") else {
        return Err(IngressTransmuterError::UnclosedTag {
            tag: "!--".to_string(),
        });
    };
    scan_xml_lite_tag(content, open + 4 + offset + 3)
}

fn scan_xml_lite_processing_instruction(
    content: &str,
    open: usize,
) -> Result<Option<XmlLiteTag<'_>>, IngressTransmuterError> {
    let Some(offset) = content[open + 2..].find("?>") else {
        return Ok(None);
    };
    scan_xml_lite_tag(content, open + 2 + offset + 2)
}

fn scan_named_xml_lite_tag(
    content: &str,
    open: usize,
    closing: bool,
) -> Result<Option<XmlLiteTag<'_>>, IngressTransmuterError> {
    let bytes = content.as_bytes();
    let tag_start = if closing { open + 2 } else { open + 1 };
    if tag_start >= bytes.len() {
        return Ok(None);
    }
    if !is_tag_name_start(bytes[tag_start]) {
        return scan_xml_lite_tag(content, open + 1);
    }

    let tag_end = xml_lite_tag_name_end(bytes, tag_start);
    let tag_name = &content[tag_start..tag_end];
    let Some(angle_offset) = content[tag_end..].find('>') else {
        return Err(IngressTransmuterError::UnclosedTag {
            tag: tag_name.to_string(),
        });
    };
    let angle_close = tag_end + angle_offset;
    let self_closing = !closing && angle_close > open && bytes[angle_close - 1] == b'/';
    Ok(Some(XmlLiteTag {
        name: tag_name,
        closing,
        self_closing,
        next_cursor: angle_close + 1,
    }))
}

fn xml_lite_tag_name_end(bytes: &[u8], tag_start: usize) -> usize {
    bytes[tag_start + 1..]
        .iter()
        .position(|byte| !is_tag_name_char(*byte))
        .map_or(bytes.len(), |offset| tag_start + 1 + offset)
}

fn apply_xml_lite_tag(
    stack: &mut Vec<String>,
    tag: XmlLiteTag<'_>,
) -> Result<(), IngressTransmuterError> {
    if tag.closing {
        return close_xml_lite_tag(stack, tag.name);
    }
    if !tag.self_closing {
        stack.push(tag.name.to_string());
    }
    Ok(())
}

fn close_xml_lite_tag(
    stack: &mut Vec<String>,
    tag_name: &str,
) -> Result<(), IngressTransmuterError> {
    match stack.pop() {
        Some(expected) if expected == tag_name => Ok(()),
        Some(expected) => Err(IngressTransmuterError::MismatchedClosingTag {
            expected,
            found: tag_name.to_string(),
        }),
        None => Err(IngressTransmuterError::UnexpectedClosingTag {
            found: tag_name.to_string(),
        }),
    }
}

fn is_tag_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_tag_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':')
}

fn should_validate_xml_lite(uri: &str) -> bool {
    let extension = uri
        .rsplit('.')
        .next()
        .map(str::trim)
        .map(str::to_ascii_lowercase);
    matches!(extension.as_deref(), Some("xml" | "xml-lite" | "xlite"))
}

#[cfg(test)]
#[path = "../../tests/unit/ingress/transmuter.rs"]
mod tests;
