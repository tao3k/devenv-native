use crate::link_graph::set_link_graph_wendao_config_override;
#[cfg(feature = "julia")]
use std::fs;
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::linked_builtin_julia_analyzer_example_config_path;

#[cfg(feature = "julia")]
pub(super) fn configure_julia_rerank_runtime_fixture()
-> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.retrieval]
mode = "hybrid"

[link_graph.retrieval.julia_rerank]
base_url = "http://127.0.0.1:8088"
route = "/rerank"
health_route = "/healthz"
schema_version = "v1"
timeout_secs = 15
service_mode = "stream"
analyzer_config_path = "{config_path}"
analyzer_strategy = "similarity_only"
vector_weight = 0.2
similarity_weight = 0.8
"#,
            config_path = linked_builtin_julia_analyzer_example_config_path()
        ),
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}
