//! Owns the Studio handlers queries symbol surface.

use serde::Deserialize;

/// Query parameters for Studio symbol search.
#[derive(Debug, Deserialize)]
pub struct SymbolSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
