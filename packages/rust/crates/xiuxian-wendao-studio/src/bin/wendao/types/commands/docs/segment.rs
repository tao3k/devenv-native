use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsSegmentArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long = "page-id")]
    pub page_id: String,
    #[arg(long = "line-start")]
    pub line_start: usize,
    #[arg(long = "line-end")]
    pub line_end: usize,
}
