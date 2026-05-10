use anyhow::{Context, Result, anyhow};
use xiuxian_io::PrjDirs;
use xiuxian_wendao::search::queries::{
    SearchQueryService,
    rest::{RestQueryRequest, query_rest_payload},
};

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{Cli, RestQueryArgs};

pub(super) async fn handle(cli: &Cli, args: &RestQueryArgs) -> Result<()> {
    let service = SearchQueryService::from_project_root(PrjDirs::project_root());
    let request: RestQueryRequest = serde_json::from_str(&args.payload).with_context(|| {
        format!(
            "failed to parse shared REST query request `{}`",
            args.payload
        )
    })?;
    let payload = query_rest_payload(&service, &request)
        .await
        .map_err(|error| anyhow!(error))
        .with_context(|| {
            format!(
                "failed to execute shared REST query request `{}`",
                args.payload
            )
        })?;
    emit(&payload, cli.output_or_json())
}
