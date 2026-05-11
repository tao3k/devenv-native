//! Runtime validation and schema for link-graph search options.

use super::enums::LinkGraphMatchStrategy;
use super::filters::{LinkGraphLinkFilter, LinkGraphRelatedFilter, LinkGraphSearchFilters};
use super::sort::LinkGraphSortTerm;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Search options for link-graph index retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSearchOptions {
    /// Matching strategy.
    pub match_strategy: LinkGraphMatchStrategy,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
    /// Result ordering terms.
    #[serde(default)]
    pub sort_terms: Vec<LinkGraphSortTerm>,
    /// Structured filters.
    #[serde(default)]
    pub filters: LinkGraphSearchFilters,
    /// Keep rows with `created_ts >= created_after`.
    #[serde(default)]
    pub created_after: Option<i64>,
    /// Keep rows with `created_ts <= created_before`.
    #[serde(default)]
    pub created_before: Option<i64>,
    /// Keep rows with `modified_ts >= modified_after`.
    #[serde(default)]
    pub modified_after: Option<i64>,
    /// Keep rows with `modified_ts <= modified_before`.
    #[serde(default)]
    pub modified_before: Option<i64>,
    /// Style anchors for CCS audit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub style_anchors: Vec<String>,
}

impl Default for LinkGraphSearchOptions {
    fn default() -> Self {
        Self {
            match_strategy: LinkGraphMatchStrategy::Fts,
            case_sensitive: false,
            sort_terms: vec![LinkGraphSortTerm::default()],
            filters: LinkGraphSearchFilters::default(),
            created_after: None,
            created_before: None,
            modified_after: None,
            modified_before: None,
            style_anchors: Vec::new(),
        }
    }
}

impl LinkGraphSearchOptions {
    /// Validate schema-equivalent constraints for runtime safety.
    ///
    /// # Errors
    ///
    /// Returns an error string when one or more query filters violate the runtime schema.
    pub fn validate(&self) -> Result<(), String> {
        validate_link_distance_filter("filters.link_to.max_distance", &self.filters.link_to)?;
        validate_link_distance_filter("filters.linked_by.max_distance", &self.filters.linked_by)?;
        validate_related_distance_filter("filters.related.max_distance", &self.filters.related)?;
        validate_related_ppr(&self.filters.related)?;
        validate_heading_level(self.filters.max_heading_level)?;
        validate_per_doc_section_cap(self.filters.per_doc_section_cap)
    }
}

fn validate_link_distance_filter(
    path: &'static str,
    filter: &Option<LinkGraphLinkFilter>,
) -> Result<(), String> {
    validate_positive_distance(path, filter.as_ref().and_then(|filter| filter.max_distance))
}

fn validate_related_distance_filter(
    path: &'static str,
    filter: &Option<LinkGraphRelatedFilter>,
) -> Result<(), String> {
    validate_positive_distance(path, filter.as_ref().and_then(|filter| filter.max_distance))
}

fn validate_positive_distance(path: &'static str, distance: Option<usize>) -> Result<(), String> {
    if distance.is_some_and(|distance| distance == 0) {
        return Err(format!(
            "link_graph search options schema violation at {path}: must be >= 1"
        ));
    }
    Ok(())
}

fn validate_related_ppr(related: &Option<LinkGraphRelatedFilter>) -> Result<(), String> {
    let Some(ppr) = related.as_ref().and_then(|filter| filter.ppr.as_ref()) else {
        return Ok(());
    };
    validate_ppr_alpha(ppr.alpha)?;
    validate_ppr_max_iter(ppr.max_iter)?;
    validate_ppr_tol(ppr.tol)
}

fn validate_ppr_alpha(alpha: Option<f64>) -> Result<(), String> {
    if alpha.is_some_and(|alpha| !(0.0..=1.0).contains(&alpha)) {
        return Err(
            "link_graph search options schema violation at filters.related.ppr.alpha: must be between 0 and 1"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_ppr_max_iter(max_iter: Option<usize>) -> Result<(), String> {
    if max_iter.is_some_and(|max_iter| max_iter == 0) {
        return Err(
            "link_graph search options schema violation at filters.related.ppr.max_iter: must be >= 1"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_ppr_tol(tol: Option<f64>) -> Result<(), String> {
    if tol.is_some_and(|tol| tol <= 0.0) {
        return Err(
            "link_graph search options schema violation at filters.related.ppr.tol: must be > 0"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_heading_level(level: Option<usize>) -> Result<(), String> {
    if level.is_some_and(|level| !(1..=6).contains(&level)) {
        return Err(
            "link_graph search options schema violation at filters.max_heading_level: must be between 1 and 6"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_per_doc_section_cap(cap: Option<usize>) -> Result<(), String> {
    if cap.is_some_and(|cap| cap == 0) {
        return Err(
            "link_graph search options schema violation at filters.per_doc_section_cap: must be >= 1"
                .to_string(),
        );
    }
    Ok(())
}
