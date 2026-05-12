use crate::zhenfa_router::native::semantic_check::docs_governance::MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE;
use crate::zhenfa_router::native::semantic_check::{FuzzySuggestionData, IssueLocation};

use super::compute_hash;
use super::{BatchFix, BatchFixMode, ByteRange, FixResult};

impl BatchFix {
    /// Create a new batch fix (v2.9 compatible).
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn new(
        issue_type: String,
        doc_path: String,
        line_number: usize,
        original_content: String,
        replacement: String,
        confidence: f32,
    ) -> Self {
        Self {
            issue_type,
            doc_path,
            line_number,
            original_content,
            replacement,
            confidence,
            source_location: None,
            mode: BatchFixMode::Replace,
            base_hash: None,
            byte_range: None,
        }
    }

    /// Create a surgical fix with byte precision (v3.0).
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn surgical(
        doc_path: String,
        line_number: usize,
        byte_range: ByteRange,
        base_hash: String,
        original_content: String,
        replacement: String,
        confidence: f32,
    ) -> Self {
        Self {
            issue_type: "invalid_observation_pattern".to_string(),
            doc_path,
            line_number,
            original_content,
            replacement,
            confidence,
            source_location: None,
            mode: BatchFixMode::Replace,
            base_hash: Some(base_hash),
            byte_range: Some(byte_range),
        }
    }

    /// Create a fix that materializes a missing file.
    #[must_use]
    pub fn create_file(doc_path: String, replacement: String, confidence: f32) -> Self {
        Self {
            issue_type: MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE.to_string(),
            doc_path,
            line_number: 1,
            original_content: String::new(),
            replacement,
            confidence,
            source_location: None,
            mode: BatchFixMode::CreateFile,
            base_hash: None,
            byte_range: None,
        }
    }

    /// Create a batch fix with source location.
    #[must_use]
    pub fn with_source(mut self, source_location: String) -> Self {
        self.source_location = Some(source_location);
        self
    }

    /// Create a batch fix with surgical precision data.
    #[must_use]
    pub fn with_surgical(mut self, byte_range: ByteRange, base_hash: String) -> Self {
        self.byte_range = Some(byte_range);
        self.base_hash = Some(base_hash);
        self
    }

    /// Generate a batch fix from a fuzzy suggestion.
    #[must_use]
    pub fn from_fuzzy_suggestion(
        doc_path: String,
        line_number: usize,
        original_content: String,
        suggestion: &FuzzySuggestionData,
    ) -> Self {
        Self {
            issue_type: "invalid_observation_pattern".to_string(),
            doc_path,
            line_number,
            original_content,
            replacement: suggestion.replacement_drawer.clone(),
            confidence: suggestion.confidence,
            source_location: suggestion.source_location.clone(),
            mode: BatchFixMode::Replace,
            base_hash: None,
            byte_range: None,
        }
    }

    /// Generate a surgical batch fix from a fuzzy suggestion with location data.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn from_fuzzy_suggestion_surgical(
        doc_path: String,
        location: &IssueLocation,
        original_content: String,
        file_content: &str,
        suggestion: &FuzzySuggestionData,
    ) -> Self {
        let base_hash = compute_hash(file_content);

        // v3.3 Immutable Cursor Algorithm
        // Find the exact byte offsets of the target line by iterating through line boundaries.
        let mut target_range: Option<ByteRange> = None;
        let mut actual_raw_line = String::new();

        let mut current_pos = 0;
        for (i, line_content) in file_content.split_inclusive('\n').enumerate() {
            let line_num = i + 1;
            let line_len = line_content.len();

            if line_num == location.line
                && (line_content.contains(":OBSERVE:")
                    || line_content.contains(&suggestion.original_pattern))
            {
                target_range = Some(ByteRange::new(current_pos, current_pos + line_len));
                actual_raw_line = line_content.to_string();
                break;
            }
            current_pos += line_len;
        }

        if target_range.is_none() && location.line > 0 {
            let mut current_pos = 0;
            for (i, line_content) in file_content.lines().enumerate() {
                let line_num = i + 1;
                let line_len = line_content.len();
                if line_num == location.line && current_pos + line_len == file_content.len() {
                    target_range = Some(ByteRange::new(current_pos, file_content.len()));
                    actual_raw_line = line_content.to_string();
                    break;
                }
                current_pos += line_len + 1;
            }
        }

        if let Some(range) = target_range {
            let mut replacement = suggestion.replacement_drawer.clone();
            if actual_raw_line.ends_with('\n') && !replacement.ends_with('\n') {
                replacement.push('\n');
            } else if actual_raw_line.ends_with("\r\n") && !replacement.ends_with("\r\n") {
                replacement.push_str("\r\n");
            }

            return Self::surgical(
                doc_path,
                location.line,
                range,
                base_hash,
                actual_raw_line,
                replacement,
                suggestion.confidence,
            )
            .with_source(suggestion.source_location.clone().unwrap_or_default());
        }

        let fallback_range = location
            .byte_range
            .map_or_else(|| ByteRange::new(0, 0), |(s, e)| ByteRange::new(s, e));

        Self::surgical(
            doc_path,
            location.line,
            fallback_range,
            base_hash,
            original_content,
            suggestion.replacement_drawer.clone(),
            suggestion.confidence,
        )
        .with_source(suggestion.source_location.clone().unwrap_or_default())
    }

    /// Check if this fix has surgical precision data.
    #[must_use]
    pub fn is_surgical(&self) -> bool {
        self.base_hash.is_some() && self.byte_range.is_some()
    }

    /// Check if this fix creates a new file.
    #[must_use]
    pub const fn is_create_file(&self) -> bool {
        matches!(self.mode, BatchFixMode::CreateFile)
    }

    /// Apply the fix using surgical precision (v3.1).
    ///
    /// # Arguments
    ///
    /// * `content` - The file content to modify
    ///
    /// # Returns
    ///
    /// - `FixResult::Success` if the fix was applied
    /// - `FixResult::OutOfBounds` if byte range exceeds file size
    /// - `FixResult::ContentMismatch` if content at range doesn't match expected
    pub fn apply_surgical(&self, content: &mut String) -> FixResult {
        if self.is_create_file() {
            content.clear();
            content.push_str(&self.replacement);
            return FixResult::Success;
        }

        let Some(range) = self.byte_range else {
            return self.apply_legacy(content);
        };

        let content_bytes = content.as_bytes();
        if !range.is_valid_for(content_bytes.len()) {
            return FixResult::OutOfBounds {
                range,
                file_size: content_bytes.len(),
            };
        }

        let actual_at_range = range.extract(content).unwrap_or("");
        if actual_at_range != self.original_content {
            return FixResult::ContentMismatch {
                expected: self.original_content.clone(),
                actual: actual_at_range.to_string(),
            };
        }

        content.replace_range(range.start..range.end, &self.replacement);
        FixResult::Success
    }

    /// Apply the fix using legacy string search (v2.9).
    fn apply_legacy(&self, content: &mut String) -> FixResult {
        let Some(start_pos) = content.find(&self.original_content) else {
            return FixResult::ContentMismatch {
                expected: self.original_content.clone(),
                actual: "(not found in document)".to_string(),
            };
        };

        let end_pos = start_pos + self.original_content.len();
        content.replace_range(start_pos..end_pos, &self.replacement);
        FixResult::Success
    }

    /// Apply the fix to a document content (v2.9 compatible).
    ///
    /// # Errors
    ///
    /// Returns an error when the target byte range is invalid or the current file content no
    /// longer matches the expected original content for this fix.
    pub fn apply(&self, content: &mut String) -> Result<(), String> {
        match self.apply_surgical(content) {
            FixResult::Success => Ok(()),
            other => Err(other.to_string()),
        }
    }

    /// Preview the fix without applying it.
    ///
    /// # Errors
    ///
    /// Returns the same verification error as [`BatchFix::apply_surgical`] when the replacement
    /// cannot be applied to the provided content.
    pub fn preview(&self, content: &str) -> Result<String, FixResult> {
        let mut content = content.to_string();
        match self.apply_surgical(&mut content) {
            FixResult::Success => Ok(content),
            other => Err(other),
        }
    }
}
