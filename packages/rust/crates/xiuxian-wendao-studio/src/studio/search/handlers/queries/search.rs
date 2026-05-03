#[cfg(test)]
use serde::Deserialize;

#[cfg(test)]
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(alias = "query")]
    pub q: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
    #[cfg(test)]
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}
