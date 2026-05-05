use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReferenceSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
