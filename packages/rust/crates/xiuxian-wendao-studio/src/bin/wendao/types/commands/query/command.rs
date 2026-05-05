#[derive(Debug, clap::Subcommand, Clone)]
pub(crate) enum QueryCommand {
    /// Execute a GraphQL query against the shared query system.
    Graphql(super::GraphqlQueryArgs),
    /// Execute a REST-style query against the shared query system.
    Rest(super::RestQueryArgs),
    /// Execute a SQL query against the shared query system.
    Sql(super::SqlQueryArgs),
}

#[cfg(test)]
pub(crate) fn query(command: QueryCommand) -> crate::bin_support::wendao::types::Command {
    crate::bin_support::wendao::types::Command::Query { command }
}
