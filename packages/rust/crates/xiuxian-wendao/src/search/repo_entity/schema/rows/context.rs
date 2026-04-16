use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::analyzers::compute_repository_saliency;
use crate::analyzers::{RepoBacklinkItem, RepositoryAnalysisOutput};
use crate::analyzers::{
    documents_backlink_lookup, example_relation_lookup, infer_ecosystem, projection_page_lookup,
};

pub(crate) struct RepoEntityContext<'a> {
    pub(crate) repo_id: &'a str,
    pub(crate) analysis: &'a RepositoryAnalysisOutput,
    pub(crate) backlink_lookup: BTreeMap<String, Vec<RepoBacklinkItem>>,
    pub(crate) projection_lookup: BTreeMap<String, Vec<String>>,
    pub(crate) saliency_map: HashMap<String, f64>,
    pub(crate) example_relations: BTreeSet<(String, String)>,
    pub(crate) ecosystem: &'static str,
}

impl<'a> RepoEntityContext<'a> {
    pub(crate) fn new(repo_id: &'a str, analysis: &'a RepositoryAnalysisOutput) -> Self {
        Self {
            repo_id,
            analysis,
            backlink_lookup: documents_backlink_lookup(&analysis.relations, &analysis.docs),
            projection_lookup: projection_page_lookup(analysis),
            saliency_map: compute_repository_saliency(analysis),
            example_relations: example_relation_lookup(&analysis.relations),
            ecosystem: infer_ecosystem(repo_id),
        }
    }

    pub(crate) fn module_path(&self, module_id: &str) -> Option<&str> {
        self.analysis
            .modules
            .iter()
            .find(|module| module.module_id == module_id)
            .map(|module| module.path.as_str())
    }
}
