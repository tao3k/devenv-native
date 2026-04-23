use std::fs;
use std::io;
use std::sync::Arc;

use xiuxian_llm::llm::{OpenAICompatibleClient, OpenAIWireApi};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::manifest_requires_llm;
use xiuxian_qianji::runtime_config::{
    resolve_qianji_runtime_checkpoint_config, resolve_qianji_runtime_llm_config,
};
use xiuxian_qianji::{QianjiCompiler, QianjiLlmClient, QianjiScheduler};
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
    let mut context: serde_json::Value = serde_json::from_str(context_json).map_err(|error| {
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
    let llm_runtime = if requires_llm {
        let resolved = resolve_qianji_runtime_llm_config().map_err(|error| {
            io::Error::other(format!(
                "Failed to resolve Qianji runtime config from qianji.toml: {error}"
            ))
        })?;
        inject_llm_model_fallback_if_missing(&mut context, &resolved.model);
        Some(resolved)
    } else {
        None
    };

    let checkpoint_runtime = resolve_qianji_runtime_checkpoint_config().map_err(|error| {
        io::Error::other(format!(
            "Failed to resolve Qianji checkpoint runtime config from qianji.toml: {error}"
        ))
    })?;

    println!("Initializing Qianji Engine on: {repo_path}");
    if let Some(runtime) = llm_runtime.as_ref() {
        println!(
            "Resolved Qianji LLM runtime config: model='{}', base_url='{}', api_key_env='{}', wire_api='{}'",
            runtime.model, runtime.base_url, runtime.api_key_env, runtime.wire_api
        );
    } else {
        println!("Manifest has no llm nodes; skipping Qianji LLM runtime initialization.");
    }
    println!(
        "Resolved Qianji checkpoint runtime config: valkey_url='{}'",
        checkpoint_runtime.valkey_url
    );

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
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = PersonaRegistry::with_builtins();
    let llm_client: Option<Arc<QianjiLlmClient>> = llm_runtime.as_ref().map(|runtime| {
        Arc::new(OpenAICompatibleClient {
            api_key: runtime.api_key.clone(),
            base_url: runtime.base_url.clone(),
            wire_api: OpenAIWireApi::parse(Some(runtime.wire_api.as_str())),
            http: reqwest::Client::new(),
        }) as Arc<QianjiLlmClient>
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), llm_client);
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

fn inject_llm_model_fallback_if_missing(context: &mut serde_json::Value, default_model: &str) {
    let Some(map) = context.as_object_mut() else {
        return;
    };

    let has_explicit_model = map
        .get("llm_model")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let has_fallback_model = map
        .get("llm_model_fallback")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if has_explicit_model || has_fallback_model {
        return;
    }

    map.insert(
        "llm_model_fallback".to_string(),
        serde_json::Value::String(default_model.to_string()),
    );
}
