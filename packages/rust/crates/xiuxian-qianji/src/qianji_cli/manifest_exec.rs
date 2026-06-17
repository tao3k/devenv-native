use std::fs;
use std::io;
use std::sync::Arc;

use crate::manifest_requires_llm;
use crate::runtime_config::resolve_qianji_runtime_checkpoint_config;
use crate::{QianjiCompiler, QianjiScheduler};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
#[cfg(feature = "wendao-integration")]
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(crate) async fn run_manifest_execution(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = &args[1];
    let manifest_path = &args[2];
    let context_json = &args[3];
    let session_id = args.get(4).cloned();

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        io::Error::other(format!(
            "Failed to read manifest file at {manifest_path}: {error}"
        ))
    })?;
    let context: serde_json::Value = serde_json::from_str(context_json).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse context_json as valid JSON: {error}"),
        )
    })?;

    let requires_llm = manifest_requires_llm(&manifest_toml).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to inspect manifest for llm requirements: {error}"),
        )
    })?;
    if requires_llm {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "manifest execution contains llm nodes; local Qianji LLM execution is retired, use marlin-agent-core or an external service adapter",
        )));
    }

    let checkpoint_runtime = resolve_qianji_runtime_checkpoint_config().map_err(|error| {
        io::Error::other(format!(
            "Failed to resolve Qianji checkpoint runtime config from qianji.toml: {error}"
        ))
    })?;

    println!("Initializing Qianji Engine on: {repo_path}");
    println!("Manifest has no llm nodes; skipping Qianji LLM runtime initialization.");
    println!(
        "Resolved Qianji checkpoint runtime config: valkey_url='{}'",
        checkpoint_runtime.valkey_url
    );

    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = PersonaRegistry::with_builtins();

    #[cfg(feature = "wendao-integration")]
    let compiler = {
        let index = Arc::new(match LinkGraphIndex::build(std::path::Path::new(repo_path)) {
            Ok(index) => index,
            Err(primary_error) => {
                LinkGraphIndex::build(std::env::temp_dir().as_path()).map_err(|fallback_error| {
                    io::Error::other(format!(
                        "Failed to build LinkGraph index at repo path ({primary_error}); fallback temp index also failed ({fallback_error})"
                    ))
                })?
            }
        });
        QianjiCompiler::new(index, orchestrator, Arc::new(registry))
    };
    #[cfg(not(feature = "wendao-integration"))]
    let compiler = QianjiCompiler::new(orchestrator, Arc::new(registry));

    let engine = compiler.compile(&manifest_toml)?;
    let scheduler = QianjiScheduler::new(engine);

    println!("Executing Context: {context_json}");

    let result = scheduler
        .run_with_checkpoint(context, session_id, Some(checkpoint_runtime.valkey_url))
        .await?;

    println!("\n=== Final Qianji Execution Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
