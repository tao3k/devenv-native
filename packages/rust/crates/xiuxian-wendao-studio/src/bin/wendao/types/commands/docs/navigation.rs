use clap::Args;

use crate::bin_support::wendao::types::ProjectionPageKindArg;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsNavigationArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long = "page-id")]
    pub page_id: String,
    #[arg(long = "node-id")]
    pub node_id: Option<String>,
    #[arg(long = "family-kind", value_enum)]
    pub family_kind: Option<ProjectionPageKindArg>,
    #[arg(long, default_value_t = 5)]
    pub related_limit: usize,
    #[arg(long, default_value_t = 3)]
    pub family_limit: usize,
}
