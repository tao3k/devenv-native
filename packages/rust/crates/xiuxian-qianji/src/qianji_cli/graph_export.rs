use std::fs;
use std::sync::Arc;

use crate::layout::{QgsTheme, QianjiLayoutEngine, generate_bpmn_xml};
use crate::{QianjiCompiler, QianjiLlmClient};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(crate) fn handle_graph_export(
    manifest_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Qianji Graph from: {manifest_path}");

    let manifest_toml = fs::read_to_string(manifest_path)?;
    let index = Arc::new(LinkGraphIndex::build(std::env::temp_dir().as_path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Visualizer".into(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let llm_client: Option<Arc<QianjiLlmClient>> = Some(Arc::new(NoopLlmClient));

    let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
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

struct NoopLlmClient;

#[async_trait::async_trait]
impl xiuxian_llm::llm::LlmClient for NoopLlmClient {
    async fn chat(
        &self,
        _request: xiuxian_llm::llm::ChatRequest,
    ) -> xiuxian_llm::llm::LlmResult<String> {
        Ok("noop".into())
    }

    async fn chat_stream(
        &self,
        _request: xiuxian_llm::llm::ChatRequest,
    ) -> xiuxian_llm::llm::LlmResult<xiuxian_llm::llm::client::ChatStream> {
        use futures::stream;
        Ok(Box::pin(stream::iter(vec![Ok("noop".to_string())])))
    }
}
