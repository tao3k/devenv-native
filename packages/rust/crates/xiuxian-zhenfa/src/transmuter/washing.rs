//! Lightweight content washing for Spider ingress.

use thiserror::Error;

use crate::ZhenfaXmlLiteTagName;
use crate::xml_lite::{extract_tag_f32, extract_tag_value};

/// Structural validation failures detected during ingress washing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ZhenfaTransmuterError {
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
        tag: ZhenfaXmlLiteTagName,
    },
}

impl ZhenfaTransmuterError {
    /// Returns one LLM-safe semantic summary of the error.
    #[must_use]
    pub fn llm_safe_message(&self) -> &'static str {
        match self {
            Self::NullByteDetected => {
                "content contains unsupported control characters; clean the payload and retry"
            }
            Self::MismatchedClosingTag { .. }
            | Self::UnexpectedClosingTag { .. }
            | Self::UnclosedTag { .. } => {
                "content has malformed XML-Lite structure; ensure all tags are balanced"
            }
        }
    }
}

/// Failures for content washing plus structural validation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ZhenfaResolveAndWashError {
    /// The supplied content was empty after trimming.
    #[error("semantic resource URI `{uri}` could not be resolved")]
    ResourceNotFound {
        /// Canonical resource URI.
        uri: String,
    },
    /// Structural validation failed after content washing.
    #[error(transparent)]
    Transmuter(#[from] ZhenfaTransmuterError),
}

/// Shared transmutation routines for XML-lite validation and normalization.
#[derive(Debug, Default, Clone, Copy)]
pub struct ZhenfaTransmuter;

impl ZhenfaTransmuter {
    /// Validate XML-lite structure in-place.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaTransmuterError`] when the payload is malformed.
    pub fn validate_structure(content: &str) -> Result<(), ZhenfaTransmuterError> {
        validate_structure(content)
    }

    /// Normalize payload for LLM consumption.
    #[must_use]
    pub fn refine_for_llm(content: &str) -> String {
        refine_for_llm(content)
    }

    /// Validate XML-lite structure and return the refined payload.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaTransmuterError`] when the payload is malformed.
    pub fn validate_and_refine(content: &str) -> Result<String, ZhenfaTransmuterError> {
        let refined = refine_for_llm(content);
        validate_structure(&refined)?;
        Ok(refined)
    }

    /// Resolve one already-loaded payload and apply lightweight washing.
    ///
    /// # Errors
    ///
    /// Returns [`ZhenfaResolveAndWashError::ResourceNotFound`] when the resolver yields no text.
    /// Returns [`ZhenfaResolveAndWashError::Transmuter`] when XML-Lite validation fails.
    pub fn resolve_and_wash<F>(uri: &str, resolver: F) -> Result<String, ZhenfaResolveAndWashError>
    where
        F: FnOnce(&str) -> Option<String>,
    {
        let canonical_uri = uri.trim();
        let raw_content =
            resolver(canonical_uri).ok_or_else(|| ZhenfaResolveAndWashError::ResourceNotFound {
                uri: canonical_uri.to_string(),
            })?;
        let refined = refine_for_llm(raw_content.as_str());
        if should_validate_xml_lite(canonical_uri) {
            validate_structure(refined.as_str())?;
        }
        Ok(refined)
    }

    /// Check for semantic integrity of reference anchors.
    #[must_use]
    pub fn check_semantic_integrity(content: &str) -> bool {
        check_semantic_integrity(content)
    }

    /// Extract text content of the first `<tag>...</tag>` block.
    #[must_use]
    pub fn get_tag_value<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
        extract_tag_value(text, tag)
    }

    /// Parse the first `<tag>...</tag>` block as `f32`.
    #[must_use]
    pub fn get_tag_f32(text: &str, tag: &str) -> Option<f32> {
        extract_tag_f32(text, tag)
    }
}

/// Check for semantic integrity of reference anchors.
#[must_use]
pub fn check_semantic_integrity(content: &str) -> bool {
    let mut cursor = 0usize;
    while let Some(offset) = content[cursor..].find("[[references/") {
        let link_start = cursor + offset + 2;
        let rest = &content[link_start..];
        let Some(end) = rest.find("]]") else {
            return false;
        };
        let link = &rest[..end];
        if !link.contains('#') {
            return false;
        }
        cursor = link_start + end + 2;
    }
    true
}

/// Normalize payload for LLM consumption.
#[must_use]
pub fn refine_for_llm(content: &str) -> String {
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

/// Validate XML-lite structure.
///
/// # Errors
///
/// Returns [`ZhenfaTransmuterError`] when the payload is malformed.
pub fn validate_structure(content: &str) -> Result<(), ZhenfaTransmuterError> {
    if content.contains('\0') {
        return Err(ZhenfaTransmuterError::NullByteDetected);
    }
    XmlLiteStructureValidator::new(content).validate()
}

struct XmlLiteStructureValidator<'a> {
    content: &'a str,
    bytes: &'a [u8],
    cursor: usize,
    stack: Vec<String>,
}

impl<'a> XmlLiteStructureValidator<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            bytes: content.as_bytes(),
            cursor: 0,
            stack: Vec::new(),
        }
    }

    fn validate(mut self) -> Result<(), ZhenfaTransmuterError> {
        while self.cursor < self.bytes.len() {
            self.scan_next_token()?;
        }
        self.finish()
    }

    fn scan_next_token(&mut self) -> Result<(), ZhenfaTransmuterError> {
        if self.bytes[self.cursor] != b'<' {
            self.cursor += 1;
            return Ok(());
        }
        if self.cursor + 1 >= self.bytes.len() {
            self.cursor = self.bytes.len();
            return Ok(());
        }
        match self.bytes[self.cursor + 1] {
            b'!' => self.scan_bang_token(),
            b'?' => {
                self.scan_processing_instruction();
                Ok(())
            }
            _ => self.scan_tag_token(),
        }
    }

    fn scan_bang_token(&mut self) -> Result<(), ZhenfaTransmuterError> {
        if self.content[self.cursor..].starts_with("<!--") {
            if let Some(offset) = self.content[self.cursor + 4..].find("-->") {
                self.cursor = self.cursor + 4 + offset + 3;
                return Ok(());
            }
            return Err(ZhenfaTransmuterError::UnclosedTag {
                tag: ZhenfaXmlLiteTagName::from("!--"),
            });
        }
        self.cursor += 1;
        Ok(())
    }

    fn scan_processing_instruction(&mut self) {
        if let Some(offset) = self.content[self.cursor + 2..].find("?>") {
            self.cursor = self.cursor + 2 + offset + 2;
        } else {
            self.cursor = self.bytes.len();
        }
    }

    fn scan_tag_token(&mut self) -> Result<(), ZhenfaTransmuterError> {
        let closing = self.bytes[self.cursor + 1] == b'/';
        let tag_start = if closing {
            self.cursor + 2
        } else {
            self.cursor + 1
        };
        if tag_start >= self.bytes.len() {
            self.cursor = self.bytes.len();
            return Ok(());
        }
        if !is_tag_name_start(self.bytes[tag_start]) {
            self.cursor += 1;
            return Ok(());
        }

        let tag_end = self.scan_tag_name_end(tag_start);
        let tag_name = &self.content[tag_start..tag_end];
        let Some(angle_close) = self.scan_angle_close(tag_end) else {
            return Err(ZhenfaTransmuterError::UnclosedTag {
                tag: ZhenfaXmlLiteTagName::from(tag_name),
            });
        };
        self.apply_tag(tag_name, closing, angle_close)?;
        self.cursor = angle_close + 1;
        Ok(())
    }

    fn scan_tag_name_end(&self, tag_start: usize) -> usize {
        let mut tag_end = tag_start + 1;
        while tag_end < self.bytes.len() && is_tag_name_char(self.bytes[tag_end]) {
            tag_end += 1;
        }
        tag_end
    }

    fn scan_angle_close(&self, tag_end: usize) -> Option<usize> {
        let mut angle_close = tag_end;
        while angle_close < self.bytes.len() && self.bytes[angle_close] != b'>' {
            angle_close += 1;
        }
        (angle_close < self.bytes.len()).then_some(angle_close)
    }

    fn apply_tag(
        &mut self,
        tag_name: &str,
        closing: bool,
        angle_close: usize,
    ) -> Result<(), ZhenfaTransmuterError> {
        let self_closing =
            !closing && angle_close > self.cursor && self.bytes[angle_close - 1] == b'/';
        if closing {
            return self.close_tag(tag_name);
        }
        if !self_closing {
            self.stack.push(tag_name.to_string());
        }
        Ok(())
    }

    fn close_tag(&mut self, tag_name: &str) -> Result<(), ZhenfaTransmuterError> {
        match self.stack.pop() {
            Some(expected) if expected == tag_name => Ok(()),
            Some(expected) => Err(ZhenfaTransmuterError::MismatchedClosingTag {
                expected,
                found: tag_name.to_string(),
            }),
            None => Err(ZhenfaTransmuterError::UnexpectedClosingTag {
                found: tag_name.to_string(),
            }),
        }
    }

    fn finish(mut self) -> Result<(), ZhenfaTransmuterError> {
        if let Some(tag) = self.stack.pop() {
            return Err(ZhenfaTransmuterError::UnclosedTag { tag: tag.into() });
        }
        Ok(())
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
