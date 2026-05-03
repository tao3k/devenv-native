use anyhow::{Context, Result, anyhow};
use xiuxian_io::PrjDirs;
use xiuxian_wendao::search::queries::{SearchQueryService, graphql::query_graphql_payload};

use crate::bin_support::wendao::helpers::emit;
use crate::bin_support::wendao::types::{Cli, GraphqlQueryArgs};

pub(super) async fn handle(cli: &Cli, args: &GraphqlQueryArgs) -> Result<()> {
    let service = SearchQueryService::from_project_root(PrjDirs::project_root());
    let payload = query_graphql_payload(&service, &args.document)
        .await
        .map_err(|error| anyhow!(error))
        .with_context(|| format!("failed to execute shared GraphQL query `{}`", args.document))?;
    emit(&payload, cli.output_or_json())
}
