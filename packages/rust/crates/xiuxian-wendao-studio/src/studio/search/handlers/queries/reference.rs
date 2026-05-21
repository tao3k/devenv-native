//! Owns the Studio handlers queries reference surface.

use serde::Deserialize;

/// Query parameters for Studio reference search.
#[derive(Debug, Deserialize)]
pub struct ReferenceSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
