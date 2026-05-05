use clap::Args;

#[derive(Debug, Args, Clone)]
pub(crate) struct SqlQueryArgs {
    /// SQL statement executed against the request-scoped query surface.
    #[arg(long = "query", short = 'q', value_name = "SQL")]
    pub(crate) query: String,
}
