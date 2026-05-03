use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};

use super::response::load_ast_search_response;
use crate::studio::{GatewayState, StudioApiError};
use xiuxian_wendao::search::contracts::AstSearchResponse;

#[cfg(test)]
use crate::studio::search::handlers::queries::AstSearchQuery;

#[cfg(test)]
pub async fn search_ast(
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<AstSearchQuery>,
) -> Result<Json<AstSearchResponse>, StudioApiError> {
    let response = load_ast_search_response(state.as_ref(), query).await?;
    Ok(Json(response))
}
