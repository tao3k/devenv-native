//! Bpmn runtime loader surface for `xiuxian-qianji`.

use super::error::BpmnOrchestrationError;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnBundleSnapshot, BpmnPackage, BpmnParseOptions, BpmnSourceFile, DmnSourceFile,
    parse_bpmn_bundle,
};

/// Loads one bounded BPMN+DMN package from filesystem paths using default
/// parser options.
///
/// # Errors
///
/// Returns [`BpmnOrchestrationError`] when one source file cannot be read or
/// when the engine rejects the resulting bundle.
pub fn load_bpmn_package_from_files(
    bpmn_path: impl AsRef<Path>,
    dmn_paths: &[PathBuf],
) -> Result<Arc<BpmnPackage>, BpmnOrchestrationError> {
    load_bpmn_package_from_files_with_options(bpmn_path, dmn_paths, &BpmnParseOptions::default())
}

/// Loads one bounded BPMN+DMN package from filesystem paths with explicit
/// parser options.
///
/// # Errors
///
/// Returns [`BpmnOrchestrationError`] when one source file cannot be read or
/// when the engine rejects the resulting bundle.
pub fn load_bpmn_package_from_files_with_options(
    bpmn_path: impl AsRef<Path>,
    dmn_paths: &[PathBuf],
    options: &BpmnParseOptions,
) -> Result<Arc<BpmnPackage>, BpmnOrchestrationError> {
    let snapshot = BpmnBundleSnapshot::new(vec![read_bpmn_source_file(bpmn_path.as_ref())?])
        .with_dmn_sources(
            dmn_paths
                .iter()
                .map(|path| read_dmn_source_file(path.as_path()))
                .collect::<Result<Vec<_>, _>>()?,
        );
    Ok(Arc::new(parse_bpmn_bundle(&snapshot, options)?))
}

fn read_bpmn_source_file(path: &Path) -> Result<BpmnSourceFile, BpmnOrchestrationError> {
    Ok(BpmnSourceFile::new(
        path.display().to_string(),
        fs::read_to_string(path).map_err(|source| BpmnOrchestrationError::ReadBpmnSource {
            path: path.to_path_buf(),
            source,
        })?,
    ))
}

fn read_dmn_source_file(path: &Path) -> Result<DmnSourceFile, BpmnOrchestrationError> {
    Ok(DmnSourceFile::new(
        path.display().to_string(),
        fs::read_to_string(path).map_err(|source| BpmnOrchestrationError::ReadDmnSource {
            path: path.to_path_buf(),
            source,
        })?,
    ))
}
