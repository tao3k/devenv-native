//! Flight metadata emission for model route intents and decisions.

use super::constants::{
    WENDAO_ROUTE_ID_HEADER, WENDAO_ROUTE_MODALITY_HEADER, WENDAO_ROUTE_PRECISION_TIER_HEADER,
    WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER, WENDAO_ROUTE_SELECTED_MODEL_HEADER,
    WENDAO_ROUTE_SELECTED_PROVIDER_HEADER, WENDAO_ROUTE_TASK_KIND_HEADER,
};
use super::types::{WendaoModelDecision, WendaoRouteIntent};

/// Emit stable Flight metadata pairs for a route intent and decision.
#[must_use]
pub fn wendao_model_route_metadata(
    intent: &WendaoRouteIntent,
    decision: &WendaoModelDecision,
) -> Vec<(&'static str, String)> {
    vec![
        (WENDAO_ROUTE_ID_HEADER, decision.route_id.clone()),
        (
            WENDAO_ROUTE_TASK_KIND_HEADER,
            intent.task_kind.as_str().to_owned(),
        ),
        (WENDAO_ROUTE_MODALITY_HEADER, intent.modality.clone()),
        (
            WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
            decision.selected_provider.clone(),
        ),
        (
            WENDAO_ROUTE_SELECTED_MODEL_HEADER,
            decision.selected_model.clone(),
        ),
        (
            WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
            decision.selected_backend_profile.clone(),
        ),
        (
            WENDAO_ROUTE_PRECISION_TIER_HEADER,
            intent.precision_tier.clone(),
        ),
    ]
}
