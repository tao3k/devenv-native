//! Owns the Studio handlers queries ast surface.

use serde::Deserialize;

/// Query parameters for Studio AST search.
#[derive(Debug, Deserialize)]
pub struct AstSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
