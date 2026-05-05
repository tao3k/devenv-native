use crate::bin_support::wendao::helpers::emit;
use crate::bin_support::wendao::types::Cli;
use anyhow::Result;
use xiuxian_wendao::{
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState, valkey_suggested_link_decide,
    valkey_suggested_link_decisions_recent, valkey_suggested_link_log,
    valkey_suggested_link_recent, valkey_suggested_link_recent_latest,
};

pub(super) fn handle_log(cli: &Cli, request: &LinkGraphSuggestedLinkRequest) -> Result<()> {
    let row = valkey_suggested_link_log(request).map_err(anyhow::Error::msg)?;
    emit(&row, cli.output_or_json())
}

pub(super) fn handle_recent(
    cli: &Cli,
    limit: usize,
    latest: bool,
    state: Option<LinkGraphSuggestedLinkState>,
) -> Result<()> {
    let state_filter = state;
    let rows = if latest {
        valkey_suggested_link_recent_latest(limit, state_filter)
    } else {
        valkey_suggested_link_recent(limit)
    }
    .map_err(anyhow::Error::msg)?;
    let filtered = if latest || state_filter.is_none() {
        rows
    } else {
        rows.into_iter()
            .filter(|row| Some(row.promotion_state) == state_filter)
            .collect()
    };
    emit(&filtered, cli.output_or_json())
}

pub(super) fn handle_decide(
    cli: &Cli,
    suggestion_id: &str,
    target_state: LinkGraphSuggestedLinkState,
    decided_by: &str,
    reason: &str,
    decided_at_unix: Option<f64>,
) -> Result<()> {
    let request = LinkGraphSuggestedLinkDecisionRequest {
        suggestion_id: suggestion_id.to_string(),
        target_state,
        decided_by: decided_by.to_string(),
        reason: reason.to_string(),
        decided_at_unix,
    };
    let result = valkey_suggested_link_decide(&request).map_err(anyhow::Error::msg)?;
    emit(&result, cli.output_or_json())
}

pub(super) fn handle_decisions(cli: &Cli, limit: usize) -> Result<()> {
    let rows = valkey_suggested_link_decisions_recent(limit).map_err(anyhow::Error::msg)?;
    emit(&rows, cli.output_or_json())
}
