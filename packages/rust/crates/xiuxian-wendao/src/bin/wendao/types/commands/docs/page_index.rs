use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsPageIndexArgs {
    #[arg(long)]
    pub repo: String,
}
