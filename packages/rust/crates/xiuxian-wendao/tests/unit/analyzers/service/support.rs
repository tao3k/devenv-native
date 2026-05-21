use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::analyzers::PluginRegistry;
use crate::analyzers::RegisteredRepository;
use crate::analyzers::{
    AnalysisContext, PluginAnalysisOutput, RepoIntelligenceError, RepoIntelligencePlugin,
    RepoSourceFile, RepositoryAnalysisOutput, RepositoryPluginConfig, bootstrap_builtin_registry,
};
use crate::analyzers::{
    ImportKind, ImportRecord, ModuleRecord, RepoSymbolKind, RepositoryRecord, SymbolRecord,
};

#[derive(Clone)]
pub(super) struct CountingJuliaPlugin {
    pub(super) calls: Arc<AtomicUsize>,
}

impl RepoIntelligencePlugin for CountingJuliaPlugin {
    fn id(&self) -> &'static str {
        "julia-code-parser"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let module_id = format!("repo:{}:module:FixturePkg", context.repository.id);
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: module_id.clone().into(),
                qualified_name: "FixturePkg".to_string(),
                path: file.path.clone().into(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone().into(),
                symbol_id: format!("repo:{}:symbol:solve", context.repository.id).into(),
                module_id: Some(module_id.into()),
                name: "solve".to_string(),
                qualified_name: "FixturePkg.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: file.path.clone().into(),
                line_start: Some(3),
                line_end: Some(3),
                signature: Some("solve(x)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone().into(),
                name: "FixturePkg".to_string(),
                path: repository_root.display().to_string().into(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            ..RepositoryAnalysisOutput::default()
        })
    }
}

#[derive(Clone)]
pub(super) struct CountingRustPlugin {
    pub(super) calls: Arc<AtomicUsize>,
}

impl RepoIntelligencePlugin for CountingRustPlugin {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let module_id = format!("repo:{}:module:fixture", context.repository.id);
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: module_id.clone().into(),
                qualified_name: "fixture".to_string(),
                path: file.path.clone().into(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone().into(),
                symbol_id: format!("repo:{}:symbol:solve", context.repository.id).into(),
                module_id: Some(module_id.into()),
                name: "solve".to_string(),
                qualified_name: "fixture.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: file.path.clone().into(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("solve(x)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone().into(),
                name: "fixture".to_string(),
                path: repository_root.display().to_string().into(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            ..RepositoryAnalysisOutput::default()
        })
    }
}

#[derive(Clone)]
pub(super) struct CountingModelicaPlugin {
    pub(super) calls: Arc<AtomicUsize>,
}

impl RepoIntelligencePlugin for CountingModelicaPlugin {
    fn id(&self) -> &'static str {
        "modelica"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        let module_id = format!("repo:{}:module:DemoLib", context.repository.id);
        Ok(PluginAnalysisOutput {
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: module_id.clone().into(),
                qualified_name: "DemoLib".to_string(),
                path: file.path.clone().into(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone().into(),
                symbol_id: format!("repo:{}:symbol:PI", context.repository.id).into(),
                module_id: Some(module_id.into()),
                name: "PI".to_string(),
                qualified_name: "DemoLib.PI".to_string(),
                kind: RepoSymbolKind::Type,
                path: file.path.clone().into(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("model PI".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: Vec::new(),
            examples: Vec::new(),
            docs: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone().into(),
                name: "DemoLib".to_string(),
                path: repository_root.display().to_string().into(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: format!("repo:{}:module:DemoLib", context.repository.id).into(),
                qualified_name: "DemoLib".to_string(),
                path: "PI.mo".to_string().into(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone().into(),
                symbol_id: format!("repo:{}:symbol:PI", context.repository.id).into(),
                module_id: Some(format!("repo:{}:module:DemoLib", context.repository.id).into()),
                name: "PI".to_string(),
                qualified_name: "DemoLib.PI".to_string(),
                kind: RepoSymbolKind::Type,
                path: "PI.mo".to_string().into(),
                line_start: Some(1),
                line_end: Some(1),
                signature: Some("model PI".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            ..RepositoryAnalysisOutput::default()
        })
    }
}

#[derive(Clone)]
pub(super) struct CachedTargetFilePlugin {
    pub(super) repository_calls: Arc<AtomicUsize>,
    pub(super) file_calls: Arc<AtomicUsize>,
}

impl RepoIntelligencePlugin for CachedTargetFilePlugin {
    fn id(&self) -> &'static str {
        "julia-code-parser"
    }

    fn supports_repository(&self, _repository: &RegisteredRepository) -> bool {
        true
    }

    fn analyze_file(
        &self,
        _context: &AnalysisContext,
        _file: &RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        self.file_calls.fetch_add(1, Ordering::SeqCst);
        Err(RepoIntelligenceError::AnalysisFailed {
            message: "target-file analysis should reuse cached repository output".to_string(),
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        self.repository_calls.fetch_add(1, Ordering::SeqCst);
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord {
                repo_id: context.repository.id.clone().into(),
                name: "FixturePkg".to_string(),
                path: repository_root.display().to_string().into(),
                url: None,
                revision: None,
                version: None,
                uuid: None,
                dependencies: Vec::new(),
            }),
            modules: vec![ModuleRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: format!("repo:{}:module:FixturePkg", context.repository.id).into(),
                qualified_name: "FixturePkg".to_string(),
                path: "src/FixturePkg.jl".to_string().into(),
            }],
            symbols: vec![SymbolRecord {
                repo_id: context.repository.id.clone().into(),
                symbol_id: format!("repo:{}:symbol:solve", context.repository.id).into(),
                module_id: Some(format!("repo:{}:module:FixturePkg", context.repository.id).into()),
                name: "solve".to_string(),
                qualified_name: "FixturePkg.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: "src/FixturePkg.jl".to_string().into(),
                line_start: Some(3),
                line_end: Some(3),
                signature: Some("solve(x)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            }],
            imports: vec![ImportRecord {
                repo_id: context.repository.id.clone().into(),
                module_id: format!("repo:{}:module:FixturePkg", context.repository.id).into(),
                path: "src/FixturePkg.jl".to_string().into(),
                import_name: "LinearAlgebra".to_string(),
                target_package: "LinearAlgebra".to_string(),
                source_module: "LinearAlgebra".to_string(),
                kind: ImportKind::Module,
                line_start: Some(2),
                resolved_id: None,
                attributes: BTreeMap::from([(
                    "dependency_form".to_string(),
                    "qualified_import".to_string(),
                )]),
            }],
            ..RepositoryAnalysisOutput::default()
        })
    }
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

pub(super) fn bootstrap_builtin_registry_with_counting_rust_plugin(
    calls: Arc<AtomicUsize>,
) -> PluginRegistry {
    let mut registry =
        bootstrap_builtin_registry().unwrap_or_else(|error| panic!("bootstrap registry: {error}"));
    registry
        .register(CountingRustPlugin { calls })
        .unwrap_or_else(|error| panic!("register Rust plugin: {error}"));
    registry
}
