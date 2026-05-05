use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AttachmentSearchQuery {
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub ext: Vec<String>,
    #[serde(default)]
    pub kind: Vec<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}
