use crate::engine::QianjiEngine;
use crate::engine::compiler_api::QianjiCompiler;
use crate::error::QianjiError;

use super::{annotation, graph_assembly, manifest_parser, mechanism_dispatch, topology_validation};

pub(in crate::engine) fn compile_manifest(
    compiler: &QianjiCompiler,
    manifest_toml: &str,
) -> Result<QianjiEngine, QianjiError> {
    let manifest = manifest_parser::parse(manifest_toml)?;
    let mut engine = QianjiEngine::new();
    let id_to_index = graph_assembly::add_manifest_nodes(
        &mut engine,
        manifest.nodes,
        |node_def| mechanism_dispatch::build(compiler, node_def),
        annotation::node_execution_affinity,
    )?;
    graph_assembly::add_manifest_edges(&mut engine, &id_to_index, manifest.edges)?;
    topology_validation::ensure_static_acyclic(&engine)?;

    Ok(engine)
}
