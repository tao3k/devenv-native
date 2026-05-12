use crate::link_graph::{
    LinkGraphAttachment, LinkGraphAttachmentHit, LinkGraphAttachmentKind, LinkGraphIndex,
};
use std::collections::HashSet;

struct AttachmentSearchQuery {
    normalized: String,
    tokens: Vec<String>,
    case_sensitive: bool,
}

struct AttachmentSearchFilters {
    extensions: HashSet<String>,
    kinds: HashSet<LinkGraphAttachmentKind>,
}

impl LinkGraphIndex {
    /// Search extracted local attachments by query, extension, and kind filters.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn search_attachments(
        &self,
        query: &str,
        limit: usize,
        extensions: &[String],
        kinds: &[LinkGraphAttachmentKind],
        case_sensitive: bool,
    ) -> Vec<LinkGraphAttachmentHit> {
        let bounded_limit = limit.max(1);
        let search_query = AttachmentSearchQuery::new(query, case_sensitive);
        let filters = AttachmentSearchFilters::new(extensions, kinds);
        let hits = self.collect_attachment_hits(&search_query, &filters);
        ranked_attachment_hits(hits, bounded_limit)
    }

    fn collect_attachment_hits(
        &self,
        query: &AttachmentSearchQuery,
        filters: &AttachmentSearchFilters,
    ) -> Vec<LinkGraphAttachmentHit> {
        self.attachments_by_doc
            .values()
            .flat_map(|rows| rows.iter())
            .filter_map(|row| attachment_hit(row, query, filters))
            .collect()
    }
}

impl AttachmentSearchQuery {
    fn new(query: &str, case_sensitive: bool) -> Self {
        let trimmed = query.trim();
        let normalized = if case_sensitive {
            trimmed.to_string()
        } else {
            trimmed.to_lowercase()
        };
        let tokens = normalized
            .split_whitespace()
            .map(ToString::to_string)
            .collect();
        Self {
            normalized,
            tokens,
            case_sensitive,
        }
    }

    fn searchable_fields(&self, row: &LinkGraphAttachment) -> Vec<String> {
        let fields = vec![
            row.attachment_path.clone(),
            row.attachment_name.clone(),
            row.source_path.clone(),
            row.source_title.clone(),
            row.source_stem.clone(),
        ];
        if self.case_sensitive {
            return fields;
        }
        fields
            .into_iter()
            .map(|value| value.to_lowercase())
            .collect()
    }

    fn score_fields(&self, fields: &[String]) -> Option<f64> {
        if self.normalized.is_empty() {
            return Some(1.0);
        }
        let query_hit = self.query_hit(fields);
        let token_hit_count = self.token_hit_count(fields);
        if !query_hit && token_hit_count == 0 {
            return None;
        }
        Some(self.weighted_score(fields, token_hit_count))
    }

    fn query_hit(&self, fields: &[String]) -> bool {
        fields
            .iter()
            .any(|value| value.contains(self.normalized.as_str()))
    }

    fn token_hit_count(&self, fields: &[String]) -> usize {
        self.tokens
            .iter()
            .filter(|token| fields.iter().any(|value| value.contains(token.as_str())))
            .count()
    }

    fn weighted_score(&self, fields: &[String], token_hit_count: usize) -> f64 {
        let exact_name = fields
            .get(1)
            .is_some_and(|value| value == &self.normalized)
            .then_some(1.0)
            .unwrap_or(0.0);
        let path_hit = fields
            .first()
            .is_some_and(|value| value.contains(self.normalized.as_str()))
            .then_some(1.0)
            .unwrap_or(0.0);
        let token_ratio = self.token_ratio(token_hit_count);
        (exact_name * 0.5 + path_hit * 0.3 + token_ratio * 0.2).clamp(0.0, 1.0)
    }

    fn token_ratio(&self, token_hit_count: usize) -> f64 {
        if self.tokens.is_empty() {
            return 0.0;
        }
        usize_to_f64_saturating(token_hit_count) / usize_to_f64_saturating(self.tokens.len())
    }
}

impl AttachmentSearchFilters {
    fn new(extensions: &[String], kinds: &[LinkGraphAttachmentKind]) -> Self {
        Self {
            extensions: extensions
                .iter()
                .map(|value| value.trim().trim_start_matches('.').to_lowercase())
                .filter(|value| !value.is_empty())
                .collect(),
            kinds: kinds.iter().copied().collect(),
        }
    }

    fn matches(&self, row: &LinkGraphAttachment) -> bool {
        self.extension_matches(row) && self.kind_matches(row)
    }

    fn extension_matches(&self, row: &LinkGraphAttachment) -> bool {
        self.extensions.is_empty() || self.extensions.contains(&row.attachment_ext)
    }

    fn kind_matches(&self, row: &LinkGraphAttachment) -> bool {
        self.kinds.is_empty() || self.kinds.contains(&row.kind)
    }
}

fn attachment_hit(
    row: &LinkGraphAttachment,
    query: &AttachmentSearchQuery,
    filters: &AttachmentSearchFilters,
) -> Option<LinkGraphAttachmentHit> {
    if !filters.matches(row) {
        return None;
    }
    let fields = query.searchable_fields(row);
    let score = query.score_fields(&fields)?;
    Some(LinkGraphAttachmentHit {
        source_id: row.source_id.clone(),
        source_stem: row.source_stem.clone(),
        source_title: row.source_title.clone(),
        source_path: row.source_path.clone(),
        attachment_path: row.attachment_path.clone(),
        attachment_name: row.attachment_name.clone(),
        attachment_ext: row.attachment_ext.clone(),
        kind: row.kind,
        score,
        vision_snippet: vision_snippet(row),
    })
}

fn vision_snippet(row: &LinkGraphAttachment) -> Option<String> {
    row.vision_annotation.as_ref().map(|value| {
        let desc = &value.description;
        if desc.len() > 100 {
            format!("{}...", &desc[..100])
        } else {
            desc.clone()
        }
    })
}

fn ranked_attachment_hits(
    mut hits: Vec<LinkGraphAttachmentHit>,
    limit: usize,
) -> Vec<LinkGraphAttachmentHit> {
    hits.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then(left.attachment_path.cmp(&right.attachment_path))
            .then(left.source_path.cmp(&right.source_path))
    });
    hits.truncate(limit);
    hits
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
