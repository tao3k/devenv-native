use std::fs;
#[cfg(feature = "wendao-integration")]
use std::sync::Arc;

use crate::QianjiCompiler;
use crate::layout::{QgsTheme, QianjiLayoutEngine, generate_bpmn_xml};
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(crate) fn handle_graph_export(
    manifest_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Qianji Graph from: {manifest_path}");

    let manifest_toml = fs::read_to_string(manifest_path)?;

    #[cfg(feature = "wendao-integration")]
    let compiler = {
        let index = Arc::new(LinkGraphIndex::build(std::env::temp_dir().as_path())?);
        QianjiCompiler::new(index)
    };
    #[cfg(not(feature = "wendao-integration"))]
    let compiler = QianjiCompiler::new();

    let engine = compiler.compile(&manifest_toml)?;
    let layout_engine = QianjiLayoutEngine::new(QgsTheme::default());
    let layout_result = layout_engine.compute_from_engine(&engine);
    let bpmn_xml = generate_bpmn_xml(&layout_result);
    let obsidian_graph = QianjiLayoutEngine::compute_obsidian_graph(&engine);
    let obsidian_path = format!(
        "{}_obsidian.json",
        output_path.strip_suffix(".bpmn").unwrap_or(output_path)
    );

    fs::write(
        &obsidian_path,
        serde_json::to_string_pretty(&obsidian_graph)?,
    )?;
    fs::write(output_path, bpmn_xml)?;

    println!("Successfully exported BPMN XML to: {output_path}");
    println!("Successfully exported Obsidian Graph to: {obsidian_path}");
    Ok(())
}
