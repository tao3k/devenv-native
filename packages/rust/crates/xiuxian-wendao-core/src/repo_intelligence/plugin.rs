//! Plugin traits and payloads for repository-intelligence analyzers.

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::config::RegisteredRepository;
use super::errors::RepoIntelligenceError;
use super::records::{
    DiagnosticRecord, DocRecord, ExampleRecord, ImportRecord, ModuleRecord, RelationRecord,
    RepositoryRecord, SymbolRecord,
};

/// Repository-relative file payload passed to plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoSourceFile {
    /// Repository-relative path.
    pub path: String,
    /// Raw UTF-8 file contents.
    pub contents: String,
}

/// Context shared with file analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AnalysisContext {
    /// Registered repository metadata.
    pub repository: RegisteredRepository,
    /// Resolved local repository root.
    pub repository_root: std::path::PathBuf,
}

/// Context shared with relation enrichment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginLinkContext {
    /// Registered repository metadata.
    pub repository: RegisteredRepository,
    /// Resolved local repository root.
    pub repository_root: std::path::PathBuf,
    /// Normalized module records produced during analysis.
    pub modules: Vec<ModuleRecord>,
    /// Normalized symbol records produced during analysis.
    pub symbols: Vec<SymbolRecord>,
    /// Normalized example records produced during analysis.
    pub examples: Vec<ExampleRecord>,
    /// Normalized documentation records produced during analysis.
    pub docs: Vec<DocRecord>,
}

/// Output returned by plugin file analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginAnalysisOutput {
    /// Module records extracted from the file.
    pub modules: Vec<ModuleRecord>,
    /// Symbol records extracted from the file.
    pub symbols: Vec<SymbolRecord>,
    /// Import records extracted from the file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportRecord>,
    /// Example records extracted from the file.
    pub examples: Vec<ExampleRecord>,
    /// Documentation records extracted from the file.
    pub docs: Vec<DocRecord>,
    /// Analysis diagnostics emitted by the plugin.
    pub diagnostics: Vec<DiagnosticRecord>,
}

/// Output returned by repository-level analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryAnalysisOutput {
    /// Repository record assembled by the analyzer.
    pub repository: Option<RepositoryRecord>,
    /// Module records extracted from repository analysis.
    pub modules: Vec<ModuleRecord>,
    /// Symbol records extracted from repository analysis.
    pub symbols: Vec<SymbolRecord>,
    /// Import records extracted from repository analysis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportRecord>,
    /// Example records extracted from repository analysis.
    pub examples: Vec<ExampleRecord>,
    /// Documentation records extracted from repository analysis.
    pub docs: Vec<DocRecord>,
    /// Relation records extracted from repository analysis.
    pub relations: Vec<RelationRecord>,
    /// Diagnostics emitted during repository analysis.
    pub diagnostics: Vec<DiagnosticRecord>,
}

/// Trait implemented by Repo Intelligence analyzers and external extensions.
pub trait RepoIntelligencePlugin: Send + Sync {
    /// Stable plugin identifier used by repository configuration.
    fn id(&self) -> &'static str;

    /// Returns true when the plugin can analyze the given repository.
    fn supports_repository(&self, repository: &RegisteredRepository) -> bool;

    /// Analyze one source file into normalized records.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the file cannot be analyzed.
    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError>;

    /// Validate repository layout before full analysis.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails.
    fn preflight_repository(
        &self,
        _context: &AnalysisContext,
        _repository_root: &Path,
    ) -> Result<(), RepoIntelligenceError> {
        Ok(())
    }

    /// Analyze a repository into normalized records.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails.
    fn analyze_repository(
        &self,
        _context: &AnalysisContext,
        _repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        Err(RepoIntelligenceError::AnalysisFailed {
            message: "repository analysis is not implemented for this plugin".to_string(),
        })
    }

    /// Enrich cross-file relations after initial records have been collected.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when relation enrichment fails.
    fn enrich_relations(
        &self,
        _context: &PluginLinkContext,
    ) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
        Ok(Vec::new())
    }
}
