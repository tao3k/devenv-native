use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsContextArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long = "page-id")]
    pub page_id: String,
    #[arg(long = "node-id")]
    pub node_id: Option<String>,
    #[arg(long, default_value_t = 5)]
    pub related_limit: usize,
}
