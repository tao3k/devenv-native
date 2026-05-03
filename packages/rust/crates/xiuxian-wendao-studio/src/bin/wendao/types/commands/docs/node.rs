use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsNodeArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long = "page-id")]
    pub page_id: String,
    #[arg(long = "node-id")]
    pub node_id: String,
}
