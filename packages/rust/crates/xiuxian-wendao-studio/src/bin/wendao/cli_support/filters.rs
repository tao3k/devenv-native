use crate::bin_support::wendao::types::RelatedPprSubgraphModeArg;
use xiuxian_wendao::{
    LinkGraphLinkFilter, LinkGraphPprSubgraphMode, LinkGraphRelatedFilter,
    LinkGraphRelatedPprOptions, LinkGraphTagFilter,
};

pub(crate) fn build_optional_link_filter(
    seeds: &[String],
    negate: bool,
    recursive: bool,
    max_distance: Option<usize>,
) -> Option<LinkGraphLinkFilter> {
    if seeds.is_empty() {
        return None;
    }
    Some(LinkGraphLinkFilter {
        seeds: seeds.to_vec(),
        negate,
        recursive,
        max_distance,
    })
}

pub(crate) fn build_optional_related_filter(
    seeds: &[String],
    max_distance: Option<usize>,
    ppr: Option<LinkGraphRelatedPprOptions>,
) -> Option<LinkGraphRelatedFilter> {
    if seeds.is_empty() {
        return None;
    }
    Some(LinkGraphRelatedFilter {
        seeds: seeds.to_vec(),
        max_distance,
        ppr,
    })
}

pub(crate) fn build_optional_related_ppr_options(
    alpha: Option<f64>,
    max_iter: Option<usize>,
    tol: Option<f64>,
    subgraph_mode: Option<RelatedPprSubgraphModeArg>,
) -> Option<LinkGraphRelatedPprOptions> {
    if alpha.is_none() && max_iter.is_none() && tol.is_none() && subgraph_mode.is_none() {
        return None;
    }
    Some(LinkGraphRelatedPprOptions {
        alpha,
        max_iter,
        tol,
        subgraph_mode: subgraph_mode.map(|mode| match mode {
            RelatedPprSubgraphModeArg::Auto => LinkGraphPprSubgraphMode::Auto,
            RelatedPprSubgraphModeArg::Disabled => LinkGraphPprSubgraphMode::Disabled,
            RelatedPprSubgraphModeArg::Force => LinkGraphPprSubgraphMode::Force,
        }),
    })
}

pub(crate) fn build_optional_tag_filter(
    all: &[String],
    any: &[String],
    not_tags: &[String],
) -> Option<LinkGraphTagFilter> {
    if all.is_empty() && any.is_empty() && not_tags.is_empty() {
        return None;
    }
    Some(LinkGraphTagFilter {
        all: all.to_vec(),
        any: any.to_vec(),
        not_tags: not_tags.to_vec(),
    })
}
