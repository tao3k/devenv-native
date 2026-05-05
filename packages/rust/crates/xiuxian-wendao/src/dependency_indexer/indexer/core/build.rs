use super::DependencyIndexer;
use crate::dependency_indexer::indexer::files::find_files;
use crate::dependency_indexer::indexer::{
    DependencyBuildConfig, DependencyIndexResult, ExternalSymbol,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;

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
        // Load configuration
        let config_path = self.config_path.as_ref().map_or_else(
            || "packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml".to_string(),
            |path| path.to_string_lossy().to_string(),
        );

        let config = DependencyBuildConfig::load(&config_path);

        if verbose {
            log::info!(
                "Loaded config with {} dependency configs",
                config.manifests.len()
            );
        }

        // Collect all manifest paths to process
        let mut all_manifests: Vec<(String, PathBuf)> = Vec::new();

        for ext_dep in &config.manifests {
            if ext_dep.pkg_type != "rust" {
                continue;
            }

            for pattern in &ext_dep.manifests {
                let manifest_paths = find_files(pattern, &self.project_root);
                for manifest_path in manifest_paths {
                    // Extract crate name from path for ordering
                    let crate_name = manifest_path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    all_manifests.push((crate_name, manifest_path));
                }
            }
        }

        if verbose {
            log::info!("Found {} manifests to process", all_manifests.len());
        }

        // Process all manifests in parallel using rayon.
        // Collect results directly without mutex contention.
        let results: Vec<(String, String, PathBuf, Vec<ExternalSymbol>, bool, String)> =
            all_manifests
                .into_par_iter()
                .map(|(crate_name, manifest_path)| {
                    let result = Self::process_manifest_inner(&manifest_path);
                    match result {
                        Ok((name, version, path, symbols)) => {
                            (name, version, path, symbols, false, String::new())
                        }
                        Err(error) => (
                            crate_name,
                            String::new(),
                            manifest_path,
                            Vec::new(),
                            true,
                            error,
                        ),
                    }
                })
                .collect();

        let mut result = DependencyIndexResult {
            files_processed: results.len(),
            total_symbols: 0,
            errors: 0,
            crates_indexed: 0,
            error_details: Vec::new(),
        };

        for (crate_name, version, _source_path, symbols, is_error, error_msg) in results {
            if is_error {
                if verbose {
                    log::warn!("Failed to process: {crate_name} - {error_msg}");
                }
                result.errors += 1;
                result
                    .error_details
                    .push(format!("{crate_name}: {error_msg}"));
            } else {
                self.crate_versions.insert(crate_name.clone(), version);
                self.symbol_index.add_symbols(&crate_name, &symbols);
                result.total_symbols += symbols.len();
                result.crates_indexed += 1;
            }
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
}
