use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SymbolSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
