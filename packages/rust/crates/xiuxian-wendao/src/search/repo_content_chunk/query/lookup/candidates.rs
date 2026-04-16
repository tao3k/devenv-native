use std::path::Path;

use crate::gateway::studio::types::{SearchHit, StudioNavigationTarget};

use super::helpers::{infer_code_language, truncate_content_search_snippet};

#[derive(Debug, Clone)]
pub(crate) struct RepoContentChunkCandidate {
    pub path: String,
    pub language: Option<String>,
    pub line_number: usize,
    pub line_text: String,
    pub score: f64,
    pub exact_match: bool,
}

impl RepoContentChunkCandidate {
    pub(crate) fn into_search_hit(self, repo_id: &str) -> SearchHit {
        let mut tags = vec![
            repo_id.to_string(),
            "code".to_string(),
            "file".to_string(),
            "kind:file".to_string(),
        ];
        if let Some(language) = self
            .language
            .clone()
            .or_else(|| infer_code_language(self.path.as_str()))
        {
            tags.push(language.clone());
            tags.push(format!("lang:{language}"));
        }
        if self.exact_match {
            tags.push("match:exact".to_string());
        }
        let stem = Path::new(self.path.as_str())
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(self.path.as_str())
            .to_string();

        SearchHit {
            stem,
            title: Some(self.path.clone()),
            path: self.path.clone(),
            doc_type: Some("file".to_string()),
            tags,
            score: self.score,
            best_section: Some(format!(
                "{}: {}",
                self.line_number,
                truncate_content_search_snippet(self.line_text.as_str(), 140)
            )),
            match_reason: Some("repo_content_search".to_string()),
            hierarchical_uri: None,
            hierarchy: Some(self.path.split('/').map(str::to_string).collect::<Vec<_>>()),
            implicit_backlinks: None,
            implicit_backlink_items: None,
            audit_status: None,
            verification_state: None,
            saliency_score: None,
            navigation_target: Some(StudioNavigationTarget {
                path: format!("{repo_id}/{}", self.path),
                category: "repo_code".to_string(),
                project_name: Some(repo_id.to_string()),
                root_label: Some(repo_id.to_string()),
                line: Some(self.line_number),
                line_end: Some(self.line_number),
                column: None,
            }),
        }
    }
}
