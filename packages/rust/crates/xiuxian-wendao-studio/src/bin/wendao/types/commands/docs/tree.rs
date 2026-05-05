use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsTreeArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long = "page-id")]
    pub page_id: String,
}
