use crate::bin_support::wendao::types::{LinkGraphScopeArg, RelatedPprSubgraphModeArg};
use clap::Args;

#[derive(Args, Debug, Default)]
pub(crate) struct SearchCaseOptions {
    #[arg(long, default_value_t = false)]
    pub case_sensitive: bool,
}

#[derive(Args, Debug, Default)]
pub(crate) struct SearchLinkToOptions {
    #[arg(long = "link-to-negate", default_value_t = false)]
    pub link_to_negate: bool,
    #[arg(long = "link-to-recursive", default_value_t = false)]
    pub link_to_recursive: bool,
}

#[derive(Args, Debug, Default)]
pub(crate) struct SearchLinkedByOptions {
    #[arg(long = "linked-by-negate", default_value_t = false)]
    pub linked_by_negate: bool,
    #[arg(long = "linked-by-recursive", default_value_t = false)]
    pub linked_by_recursive: bool,
}

#[derive(Args, Debug, Default)]
pub(crate) struct SearchFilterFlags {
    #[arg(long = "orphan", default_value_t = false)]
    pub orphan: bool,
    #[arg(long = "tagless", default_value_t = false)]
    pub tagless: bool,
    #[arg(long = "missing-backlink", default_value_t = false)]
    pub missing_backlink: bool,
}

#[derive(Args, Debug, Default)]
pub(crate) struct SearchVerbosityOptions {
    /// Include aggregated monitor phases + bottleneck summary in response payload.
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SearchArgs {
    pub query: String,
    #[arg(short, long, default_value_t = 20)]
    pub limit: usize,
    #[arg(long = "match-strategy", default_value = "fts")]
    pub match_strategy: String,
    #[arg(long = "sort-term", value_name = "TERM", num_args = 1..)]
    pub sort_terms: Vec<String>,
    #[command(flatten)]
    pub case_options: SearchCaseOptions,
    #[arg(long = "include-path", value_name = "PATH", num_args = 1..)]
    pub include_paths: Vec<String>,
    #[arg(long = "exclude-path", value_name = "PATH", num_args = 1..)]
    pub exclude_paths: Vec<String>,
    #[arg(long = "tag-all", value_name = "TAG", num_args = 1..)]
    pub tags_all: Vec<String>,
    #[arg(long = "tag-any", value_name = "TAG", num_args = 1..)]
    pub tags_any: Vec<String>,
    #[arg(long = "tag-not", value_name = "TAG", num_args = 1..)]
    pub tags_not: Vec<String>,
    #[arg(long = "link-to", value_name = "NOTE", num_args = 1..)]
    pub link_to: Vec<String>,
    #[command(flatten)]
    pub link_to_options: SearchLinkToOptions,
    #[arg(long = "link-to-max-distance")]
    pub link_to_max_distance: Option<usize>,
    #[arg(long = "linked-by", value_name = "NOTE", num_args = 1..)]
    pub linked_by: Vec<String>,
    #[command(flatten)]
    pub linked_by_options: SearchLinkedByOptions,
    #[arg(long = "linked-by-max-distance")]
    pub linked_by_max_distance: Option<usize>,
    #[arg(long = "related", value_name = "NOTE", num_args = 1..)]
    pub related: Vec<String>,
    #[arg(long = "max-distance")]
    pub max_distance: Option<usize>,
    #[arg(long = "related-ppr-alpha", requires = "related")]
    pub related_ppr_alpha: Option<f64>,
    #[arg(long = "related-ppr-max-iter", requires = "related")]
    pub related_ppr_max_iter: Option<usize>,
    #[arg(long = "related-ppr-tol", requires = "related")]
    pub related_ppr_tol: Option<f64>,
    #[arg(long = "related-ppr-subgraph-mode", requires = "related", value_enum)]
    pub related_ppr_subgraph_mode: Option<RelatedPprSubgraphModeArg>,
    #[arg(long = "mentions-of", value_name = "PHRASE", num_args = 1..)]
    pub mentions_of: Vec<String>,
    #[arg(long = "mentioned-by-notes", value_name = "NOTE", num_args = 1..)]
    pub mentioned_by_notes: Vec<String>,
    #[command(flatten)]
    pub filter_flags: SearchFilterFlags,
    #[arg(long = "created-after")]
    pub created_after: Option<i64>,
    #[arg(long = "created-before")]
    pub created_before: Option<i64>,
    #[arg(long = "modified-after")]
    pub modified_after: Option<i64>,
    #[arg(long = "modified-before")]
    pub modified_before: Option<i64>,

    #[arg(long, value_enum)]
    pub scope: Option<LinkGraphScopeArg>,
    #[arg(long)]
    pub max_heading_level: Option<usize>,
    #[arg(long)]
    pub max_tree_hops: Option<usize>,
    #[arg(long)]
    pub collapse_to_doc: Option<bool>,
    #[arg(long)]
    pub per_doc_section_cap: Option<usize>,
    #[arg(long)]
    pub min_section_words: Option<usize>,

    /// Include provisional suggested-link rows in search response payload.
    ///
    /// Supports optional explicit value:
    /// `--include-provisional` (true), `--include-provisional=false`.
    /// When omitted, runtime config default is used.
    #[arg(
        long = "include-provisional",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub include_provisional: Option<bool>,
    /// Max provisional suggested-link rows returned in search payload.
    ///
    /// When omitted, runtime config default is used.
    #[arg(long = "provisional-limit")]
    pub provisional_limit: Option<usize>,
    #[command(flatten)]
    pub verbosity: SearchVerbosityOptions,
}
