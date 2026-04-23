use xiuxian_db_store::VectorStoreError;

use crate::analyzers::RepositoryAnalysisOutput;
use crate::search::repo_entity::schema::definitions::RepoEntityRow;
use crate::search::repo_entity::schema::rows::{
    RepoEntityContext, build_example_row, build_import_row, build_module_row, build_symbol_row,
};

/// Builds repo-entity rows from repository analysis.
pub(crate) fn rows_from_analysis(
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
) -> Result<Vec<RepoEntityRow>, VectorStoreError> {
    let context = RepoEntityContext::new(repo_id, analysis);
    let mut rows = Vec::new();

    for module in &analysis.modules {
        rows.push(build_module_row(&context, module)?);
    }

    for symbol in &analysis.symbols {
        rows.push(build_symbol_row(&context, symbol)?);
    }

    for example in &analysis.examples {
        rows.push(build_example_row(&context, example)?);
    }

    for import in &analysis.imports {
        rows.push(build_import_row(&context, import)?);
    }

    Ok(rows)
}
