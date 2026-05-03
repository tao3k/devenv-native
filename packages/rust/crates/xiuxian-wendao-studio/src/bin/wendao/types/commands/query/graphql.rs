use clap::Args;

#[derive(Debug, Args, Clone)]
pub(crate) struct GraphqlQueryArgs {
    /// GraphQL document executed against the shared query surface.
    #[arg(long = "document", short = 'd', value_name = "GRAPHQL")]
    pub(crate) document: String,
}
