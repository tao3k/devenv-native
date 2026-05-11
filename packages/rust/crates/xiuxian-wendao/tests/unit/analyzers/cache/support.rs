use std::collections::{BTreeMap, HashMap};

use crate::analyzers::RepositoryPluginConfig;
use crate::search::SearchDocumentIndex;
pub(super) use crate::test_support::linked_parser_summary::{
    ensure_linked_modelica_parser_summary_service, ensure_linked_parser_summary_service,
};

use crate::analyzers::cache::{RepositoryAnalysisCacheKey, RepositorySearchArtifacts};

pub(super) fn ok_or_panic<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn some_or_panic<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn mixed_modelica_rust_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("modelica".to_string()),
        RepositoryPluginConfig::Id("rust".to_string()),
    ]
}

pub(super) fn mixed_rust_unknown_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("rust".to_string()),
        RepositoryPluginConfig::Id("ast-grep".to_string()),
    ]
}

pub(super) fn mixed_modelica_unknown_plugin_configs() -> Vec<RepositoryPluginConfig> {
    vec![
        RepositoryPluginConfig::Id("modelica".to_string()),
        RepositoryPluginConfig::Id("ast-grep".to_string()),
    ]
}

pub(super) fn sample_analysis_key(repo_id: &str) -> RepositoryAnalysisCacheKey {
    RepositoryAnalysisCacheKey {
        repo_id: repo_id.to_string().into(),
        checkout_root: format!("/virtual/{repo_id}"),
        analysis_identity: format!("analysis:{repo_id}"),
        checkout_revision: Some("rev-1".to_string()),
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        plugin_ids: vec!["plugin-a".to_string()],
    }
}

pub(super) fn empty_artifacts() -> RepositorySearchArtifacts {
    RepositorySearchArtifacts {
        module_index: SearchDocumentIndex::new(),
        symbol_index: SearchDocumentIndex::new(),
        example_index: SearchDocumentIndex::new(),
        projected_page_index: SearchDocumentIndex::new(),
        modules_by_id: BTreeMap::default(),
        symbols_by_id: BTreeMap::default(),
        examples_by_id: BTreeMap::default(),
        example_metadata: BTreeMap::default(),
        projected_pages_by_id: HashMap::default(),
        projected_pages: Vec::new(),
    }
}
