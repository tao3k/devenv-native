use std::collections::BTreeSet;

use crate::analyzers::{PluginAnalysisOutput, RepositoryAnalysisOutput};

pub(super) fn replace_records_for_paths(
    base: &mut RepositoryAnalysisOutput,
    mut overlay: PluginAnalysisOutput,
    changed_paths: &BTreeSet<String>,
    deleted_paths: &BTreeSet<String>,
) {
    let replaced_paths = changed_paths
        .iter()
        .chain(deleted_paths.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    base.modules
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));
    base.symbols
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));
    base.imports
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));
    base.examples
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));
    base.docs
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));
    base.diagnostics
        .retain(|record| !matches_record_path(record.path.as_str(), &replaced_paths));

    base.modules.append(&mut overlay.modules);
    base.symbols.append(&mut overlay.symbols);
    base.imports.append(&mut overlay.imports);
    base.examples.append(&mut overlay.examples);
    base.docs.append(&mut overlay.docs);
    base.diagnostics.append(&mut overlay.diagnostics);
}

fn matches_record_path(path: &str, replaced_paths: &BTreeSet<String>) -> bool {
    replaced_paths
        .iter()
        .any(|candidate| path == candidate || path.starts_with(format!("{candidate}#").as_str()))
}
