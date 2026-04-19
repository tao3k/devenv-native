//! Search command execution handler.

use crate::helpers::{
    build_optional_link_filter, build_optional_related_filter, build_optional_related_ppr_options,
    build_optional_tag_filter, emit, parse_sort_terms,
};
use crate::types::{Cli, Command, LinkGraphScopeArg};
use anyhow::{Context, Result};
use xiuxian_wendao::{
    LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphScope, LinkGraphSearchFilters,
    LinkGraphSearchOptions,
};

pub(super) fn handle(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let Command::Search(args) = &cli.command else {
        unreachable!("search handler called with non-search command");
    };
    let index = index.context("link_graph index is required for search command")?;

    let sort_terms = parse_sort_terms(&args.sort_terms);
    let match_strategy = match args.match_strategy.to_lowercase().as_str() {
        "exact" => LinkGraphMatchStrategy::Exact,
        "regex" | "re" => LinkGraphMatchStrategy::Re,
        _ => LinkGraphMatchStrategy::Fts,
    };

    let base_options = LinkGraphSearchOptions {
        match_strategy,
        case_sensitive: args.case_options.case_sensitive,
        sort_terms,
        filters: LinkGraphSearchFilters {
            scope: args.scope.map(|scope| match scope {
                LinkGraphScopeArg::Mixed => LinkGraphScope::Mixed,
                LinkGraphScopeArg::DocOnly => LinkGraphScope::DocOnly,
                LinkGraphScopeArg::SectionOnly => LinkGraphScope::SectionOnly,
            }),
            max_heading_level: args.max_heading_level,
            max_tree_hops: args.max_tree_hops,
            collapse_to_doc: args.collapse_to_doc,
            include_paths: args.include_paths.clone(),
            exclude_paths: args.exclude_paths.clone(),
            mentions_of: args.mentions_of.clone(),
            mentioned_by_notes: args.mentioned_by_notes.clone(),
            tags: build_optional_tag_filter(&args.tags_all, &args.tags_any, &args.tags_not),
            link_to: build_optional_link_filter(
                &args.link_to,
                args.link_to_options.link_to_negate,
                args.link_to_options.link_to_recursive,
                args.link_to_max_distance,
            ),
            linked_by: build_optional_link_filter(
                &args.linked_by,
                args.linked_by_options.linked_by_negate,
                args.linked_by_options.linked_by_recursive,
                args.linked_by_max_distance,
            ),
            related: build_optional_related_filter(
                &args.related,
                args.max_distance,
                build_optional_related_ppr_options(
                    args.related_ppr_alpha,
                    args.related_ppr_max_iter,
                    args.related_ppr_tol,
                    args.related_ppr_subgraph_mode,
                ),
            ),
            orphan: args.filter_flags.orphan,
            tagless: args.filter_flags.tagless,
            missing_backlink: args.filter_flags.missing_backlink,
            edge_types: Vec::new(),
            per_doc_section_cap: args.per_doc_section_cap,
            min_section_words: args.min_section_words,
        },
        ..LinkGraphSearchOptions::default()
    };

    if args.verbosity.verbose {
        let planned = index.search_planned_payload_with_agentic(
            &args.query,
            args.limit,
            base_options,
            args.include_provisional,
            args.provisional_limit,
        );
        emit(&planned, cli.output_or_json())
    } else {
        let (parsed, hits) = index.search_planned(&args.query, args.limit, base_options);
        if cli.output_or_json() == crate::types::OutputFormat::Json {
            emit(&hits, cli.output_or_json())
        } else {
            println!("Query: {}", parsed.query);
            println!("Hits: {}", hits.len());
            for (i, hit) in hits.iter().enumerate() {
                println!("{}. {} ({})", i + 1, hit.title, hit.path);
            }
            Ok(())
        }
    }
}
