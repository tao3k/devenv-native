//! Dependency index build orchestration.

use super::DependencyIndexer;
use crate::dependency_indexer::indexer::files::find_files;
use crate::dependency_indexer::indexer::{
    DependencyBuildConfig, DependencyIndexResult, ExternalSymbol,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;

struct ManifestWorkItem {
    crate_name: String,
    manifest_path: PathBuf,
}

struct ProcessedManifest {
    crate_name: String,
    version: String,
    symbols: Vec<ExternalSymbol>,
    error: Option<String>,
}

impl DependencyIndexer {
    /// Load the existing index from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the cached index exists but cannot be read or parsed.
    pub fn load_index(&mut self) -> Result<(), String> {
        let cache_path = self
            .project_root
            .join(".cache/xiuxian-wendao/dependency-symbol-index.txt");
        if !cache_path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&cache_path).map_err(|error| {
            format!(
                "Failed to read cache file '{}': {error}",
                cache_path.display()
            )
        })?;
        if !self.symbol_index.deserialize(&data) {
            return Err(format!(
                "Failed to deserialize symbol index from '{}'",
                cache_path.display()
            ));
        }
        Ok(())
    }

    /// Build the dependency index with parallel crate processing.
    pub fn build(&mut self, verbose: bool) -> DependencyIndexResult {
        let config = self.load_build_config(verbose);
        let work_items = self.collect_manifest_work_items(&config);
        log_manifest_count(verbose, work_items.len());
        let results = process_manifest_work_items(work_items);
        self.apply_manifest_results(results, verbose)
    }

    fn load_build_config(&self, verbose: bool) -> DependencyBuildConfig {
        let config_path = self.config_path_string();
        let config = DependencyBuildConfig::load(&config_path);
        if verbose {
            log::info!(
                "Loaded config with {} dependency configs",
                config.manifests.len()
            );
        }
        config
    }

    fn config_path_string(&self) -> String {
        self.config_path.as_ref().map_or_else(
            || "packages/rust/crates/xiuxian-wendao/resources/config/xiuxian.toml".to_string(),
            |path| path.to_string_lossy().to_string(),
        )
    }

    fn collect_manifest_work_items(&self, config: &DependencyBuildConfig) -> Vec<ManifestWorkItem> {
        config
            .manifests
            .iter()
            .filter(|ext_dep| ext_dep.pkg_type == "rust")
            .flat_map(|ext_dep| {
                ext_dep
                    .manifests
                    .iter()
                    .flat_map(|pattern| self.collect_manifest_pattern_work_items(pattern))
            })
            .collect()
    }

    fn collect_manifest_pattern_work_items(&self, pattern: &str) -> Vec<ManifestWorkItem> {
        find_files(pattern, &self.project_root)
            .into_iter()
            .map(manifest_work_item)
            .collect()
    }

    fn apply_manifest_results(
        &mut self,
        results: Vec<ProcessedManifest>,
        verbose: bool,
    ) -> DependencyIndexResult {
        let mut result = DependencyIndexResult {
            files_processed: results.len(),
            total_symbols: 0,
            errors: 0,
            crates_indexed: 0,
            error_details: Vec::new(),
        };

        for manifest in results {
            self.apply_manifest_result(manifest, verbose, &mut result);
        }

        if verbose {
            log::info!(
                "Build complete: {} files, {} symbols, {} errors",
                result.files_processed,
                result.total_symbols,
                result.errors
            );
        }

        result
    }

    fn apply_manifest_result(
        &mut self,
        manifest: ProcessedManifest,
        verbose: bool,
        result: &mut DependencyIndexResult,
    ) {
        match manifest.error {
            Some(error) => record_manifest_error(result, verbose, &manifest.crate_name, &error),
            None => {
                self.crate_versions
                    .insert(manifest.crate_name.clone(), manifest.version);
                self.symbol_index
                    .add_symbols(&manifest.crate_name, &manifest.symbols);
                result.total_symbols += manifest.symbols.len();
                result.crates_indexed += 1;
            }
        }
    }
}

fn manifest_work_item(manifest_path: PathBuf) -> ManifestWorkItem {
    ManifestWorkItem {
        crate_name: manifest_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string(),
        manifest_path,
    }
}

fn log_manifest_count(verbose: bool, count: usize) {
    if verbose {
        log::info!("Found {count} manifests to process");
    }
}

fn process_manifest_work_items(work_items: Vec<ManifestWorkItem>) -> Vec<ProcessedManifest> {
    work_items
        .into_par_iter()
        .map(process_manifest_work_item)
        .collect()
}

fn process_manifest_work_item(work_item: ManifestWorkItem) -> ProcessedManifest {
    match DependencyIndexer::process_manifest_inner(&work_item.manifest_path) {
        Ok((name, version, _path, symbols)) => ProcessedManifest {
            crate_name: name,
            version,
            symbols,
            error: None,
        },
        Err(error) => ProcessedManifest {
            crate_name: work_item.crate_name,
            version: String::new(),
            symbols: Vec::new(),
            error: Some(error),
        },
    }
}

fn record_manifest_error(
    result: &mut DependencyIndexResult,
    verbose: bool,
    crate_name: &str,
    error: &str,
) {
    if verbose {
        log::warn!("Failed to process: {crate_name} - {error}");
    }
    result.errors += 1;
    result.error_details.push(format!("{crate_name}: {error}"));
}
