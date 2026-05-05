use clap::Args;

#[derive(Debug, Args, Clone)]
pub(crate) struct RestQueryArgs {
    /// JSON request executed against the shared REST query adapter.
    #[arg(long = "payload", short = 'p', value_name = "JSON")]
    pub(crate) payload: String,
}
