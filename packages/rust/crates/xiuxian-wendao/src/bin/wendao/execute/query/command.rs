pub(crate) async fn handle(cli: &crate::types::Cli) -> anyhow::Result<()> {
    let crate::types::Command::Query { command } = &cli.command else {
        unreachable!("query handler called with non-query command");
    };

    match command {
        crate::types::QueryCommand::Graphql(args) => super::graphql::handle(cli, args).await,
        crate::types::QueryCommand::Rest(args) => super::rest::handle(cli, args).await,
        crate::types::QueryCommand::Sql(args) => super::sql::handle(cli, args).await,
    }
}
