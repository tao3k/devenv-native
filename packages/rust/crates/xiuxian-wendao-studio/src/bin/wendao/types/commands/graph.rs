use crate::bin_support::wendao::types::RelatedPprSubgraphModeArg;
use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct TocArgs {
    #[arg(short, long, default_value_t = 100)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub(crate) struct NeighborsArgs {
    pub stem: String,
    #[arg(long, default_value = "both")]
    pub direction: String,
    #[arg(long, default_value_t = 1)]
    pub hops: usize,
    #[arg(short, long, default_value_t = 50)]
    pub limit: usize,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub(crate) struct RelatedArgs {
    pub stem: String,
    #[arg(long, default_value_t = 2)]
    pub max_distance: usize,
    #[arg(short, long, default_value_t = 20)]
    pub limit: usize,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
    #[arg(long = "ppr-alpha")]
    pub ppr_alpha: Option<f64>,
    #[arg(long = "ppr-max-iter")]
    pub ppr_max_iter: Option<usize>,
    #[arg(long = "ppr-tol")]
    pub ppr_tol: Option<f64>,
    #[arg(long = "ppr-subgraph-mode", value_enum)]
    pub ppr_subgraph_mode: Option<RelatedPprSubgraphModeArg>,
}

#[derive(Args, Debug)]
pub(crate) struct MetadataArgs {
    pub stem: String,
}

#[derive(Args, Debug)]
pub(crate) struct ResolveArgs {
    pub alias: String,
    #[arg(short, long, default_value_t = 50)]
    pub limit: usize,
}
