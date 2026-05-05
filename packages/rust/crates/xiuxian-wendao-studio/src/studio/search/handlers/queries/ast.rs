use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AstSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
