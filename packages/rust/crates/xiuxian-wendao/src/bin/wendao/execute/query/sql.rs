use anyhow::{Context, Result, anyhow};
use xiuxian_io::PrjDirs;
use xiuxian_wendao::search::queries::{SearchQueryService, sql::query_sql_payload};

use crate::helpers::emit;
use crate::types::{Cli, SqlQueryArgs};

pub(super) async fn handle(cli: &Cli, args: &SqlQueryArgs) -> Result<()> {
    let service = SearchQueryService::from_project_root(PrjDirs::project_root());
    let payload = query_sql_payload(&service, &args.query)
        .await
        .map_err(|error| anyhow!(error))
        .with_context(|| format!("failed to execute shared SQL query `{}`", args.query))?;
    emit(&payload, cli.output_or_json())
}
