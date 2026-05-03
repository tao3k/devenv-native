use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocsTocArgs {
    #[arg(long)]
    pub repo: String,
}
