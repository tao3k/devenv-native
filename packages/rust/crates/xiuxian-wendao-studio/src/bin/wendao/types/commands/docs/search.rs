use clap::Args;

use crate::bin_support::wendao::types::ProjectionPageKindArg;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsSearchArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub query: String,
    #[arg(long, value_enum)]
    pub kind: Option<ProjectionPageKindArg>,
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
}
