pub(crate) async fn handle(cli: &crate::bin_support::wendao::types::Cli) -> anyhow::Result<()> {
    let crate::bin_support::wendao::types::Command::Query { command } = &cli.command else {
        unreachable!("query handler called with non-query command");
    };

    match command {
        crate::bin_support::wendao::types::QueryCommand::Graphql(args) => {
            super::graphql::handle(cli, args).await
        }
        crate::bin_support::wendao::types::QueryCommand::Rest(args) => {
            super::rest::handle(cli, args).await
        }
        crate::bin_support::wendao::types::QueryCommand::Sql(args) => {
            super::sql::handle(cli, args).await
        }
    }
}
