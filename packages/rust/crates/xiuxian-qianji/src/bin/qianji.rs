//! Qianji (千机) - The automated execution engine binary.
//!
//! This binary provides the entrypoint for compiling manifests and executing
//! long-running agentic workflows within the Xiuxian ecosystem.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use xiuxian_config_core::{resolve_cache_home, resolve_project_root_or_cwd_from_value};
use xiuxian_llm::llm::{LlmClient, OpenAICompatibleClient, OpenAIWireApi};
use xiuxian_logging::{init, split_logging_args};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::contract_feedback::{
    QianjiContractFeedbackRun, QianjiLiveContractFeedbackOptions,
    QianjiLiveContractFeedbackRuntime, QianjiPersistedContractFeedbackRun,
    build_rest_docs_collection_context, build_rest_docs_contract_suite,
    run_and_persist_contract_feedback_flow,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_and_persist_rest_docs_contract_feedback, run_contract_feedback_flow_with_live_advisory,
    run_rest_docs_contract_feedback,
};
use xiuxian_qianji::error::QianjiError;
use xiuxian_qianji::executors::formal_audit::QianjiAdvisoryAuditExecutor;
use xiuxian_qianji::layout::{QgsTheme, QianjiLayoutEngine, generate_bpmn_xml};
use xiuxian_qianji::manifest_requires_llm;
use xiuxian_qianji::runtime_config::{
    resolve_qianji_runtime_checkpoint_config, resolve_qianji_runtime_llm_config,
};
use xiuxian_qianji::sovereign::KnowledgeStorageContractFeedbackSink;
use xiuxian_qianji::{
    QianjiCompiler, QianjiLlmClient, QianjiScheduler, WorkdirCheckReport, WorkdirDiagnostic,
    advance_workdir_step, check_flowhub, check_flowhub_scenario, check_workdir,
    classify_flowhub_dir, looks_like_flowhub_scenario_dir, looks_like_workdir_dir,
    materialize_flowhub_anchored_scenario_at_node, render_anchored_materialized_workdir,
    render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    render_wendao_docs_contract_show, render_workdir_advance, render_workdir_check_markdown,
    render_workdir_show, show_flowhub, show_flowhub_anchored_scenario, show_flowhub_graph,
    show_flowhub_scenario, show_wendao_docs_contract, show_workdir,
};
use xiuxian_testing::{
    AdvisoryAuditPolicy, CollectionContext, ContractReport, ContractRunConfig, FindingSeverity,
    NoopAdvisoryAuditExecutor,
};
use xiuxian_wendao::link_graph::LinkGraphIndex;

const DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME: &str = "contract_feedback";
const REST_DOCS_PACK_ID: &str = "rest_docs";

#[derive(Debug, Clone, PartialEq, Eq)]
enum DirCliCommand {
    Show { target: ShowCliTarget },
    Check { dir: PathBuf },
    Materialize { target: MaterializeCliTarget },
    Advance { dir: PathBuf, to: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShowCliTarget {
    Dir(PathBuf),
    Graph(PathBuf),
    Contract(String),
    AnchoredScenario {
        anchor: PathBuf,
        scenario: String,
        dir: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MaterializeCliTarget {
    AnchoredScenario {
        anchor: PathBuf,
        scenario: String,
        dir: PathBuf,
        current_node: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirCliOutput {
    rendered: String,
    exit_code: i32,
}

#[derive(Debug, Clone, PartialEq)]
enum ContractFeedbackCliCommand {
    RestDocs(RestDocsCliCommand),
}

#[derive(Debug, Clone, PartialEq)]
struct RestDocsCliCommand {
    openapi_path: PathBuf,
    workspace_root: Option<PathBuf>,
    storage_path: Option<PathBuf>,
    table_name: String,
    no_persist: bool,
    live_advisory: bool,
    roles: Vec<String>,
    model: Option<String>,
    temperature: Option<f32>,
    cognitive_early_halt_threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ContractFeedbackStorageOutput {
    storage_path: String,
    table_name: String,
}

#[derive(Debug, Serialize)]
struct ContractFeedbackCliOutput {
    openapi_path: PathBuf,
    workspace_root: PathBuf,
    live_advisory: bool,
    advisory_roles: Vec<String>,
    report: ContractReport,
    knowledge_entry_ids: Vec<String>,
    persisted_entry_ids: Vec<String>,
    storage: Option<ContractFeedbackStorageOutput>,
}

/// Main entry point for the Qianji execution engine.
///
/// # Errors
/// Returns an error if environment resolution, compilation, or execution fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = env::args().collect();
    let (log_settings, args) = split_logging_args(&raw_args);
    init("xiuxian_qianji", &log_settings)?;

    if let Some(command) = parse_dir_command(&args)? {
        return handle_dir_command(command);
    }

    // Support "graph" subcommand: qianji graph <manifest_path> <output_path>
    if args.len() >= 4 && args[1] == "graph" {
        return handle_graph_export(&args[2], &args[3]);
    }

    if let Some(command) = parse_contract_feedback_command(&args)? {
        return handle_contract_feedback_command(command).await;
    }

    if args.len() < 4 {
        print_qianji_usage();
        std::process::exit(1);
    }

    run_manifest_execution(&args).await
}

async fn run_manifest_execution(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = &args[1];
    let manifest_path = &args[2];
    let context_json = &args[3];
    let session_id = args.get(4).cloned();

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|e| {
        io::Error::other(format!(
            "Failed to read manifest file at {manifest_path}: {e}"
        ))
    })?;

    let mut context: serde_json::Value = serde_json::from_str(context_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse context_json as valid JSON: {e}"),
        )
    })?;

    let requires_llm = manifest_requires_llm(&manifest_toml).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to inspect manifest for llm requirements: {e}"),
        )
    })?;
    let llm_runtime = if requires_llm {
        let resolved = resolve_qianji_runtime_llm_config().map_err(|e| {
            io::Error::other(format!(
                "Failed to resolve Qianji runtime config from qianji.toml: {e}"
            ))
        })?;
        inject_llm_model_fallback_if_missing(&mut context, &resolved.model);
        Some(resolved)
    } else {
        None
    };

    let checkpoint_runtime = resolve_qianji_runtime_checkpoint_config().map_err(|e| {
        io::Error::other(format!(
            "Failed to resolve Qianji checkpoint runtime config from qianji.toml: {e}"
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

    let index = Arc::new(
        match LinkGraphIndex::build(std::path::Path::new(repo_path)) {
            Ok(index) => index,
            Err(primary_error) => {
                LinkGraphIndex::build(std::env::temp_dir().as_path()).map_err(|fallback_error| {
                    io::Error::other(format!(
                        "Failed to build LinkGraph index at repo path ({primary_error}); \
fallback temp index also failed ({fallback_error})"
                    ))
                })?
            }
        },
    );

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

fn handle_dir_command(command: DirCliCommand) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_dir_command(command)?;
    println!("{}", output.rendered);
    if output.exit_code == 0 {
        Ok(())
    } else {
        std::process::exit(output.exit_code);
    }
}

fn run_dir_command(command: DirCliCommand) -> Result<DirCliOutput, QianjiError> {
    match command {
        DirCliCommand::Show { target } => run_show_command(&target),
        DirCliCommand::Check { dir } => run_check_dir_command(&dir),
        DirCliCommand::Materialize { target } => run_materialize_command(&target),
        DirCliCommand::Advance { dir, to } => run_advance_command(&dir, &to),
    }
}

fn run_show_command(target: &ShowCliTarget) -> Result<DirCliOutput, QianjiError> {
    match target {
        ShowCliTarget::Dir(dir) => run_show_dir_command(dir),
        ShowCliTarget::Graph(graph) => run_show_graph_command(graph),
        ShowCliTarget::Contract(contract_name) => run_show_contract_command(contract_name),
        ShowCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
        } => run_show_anchored_scenario_command(anchor, scenario, dir.as_deref()),
    }
}

fn run_show_dir_command(dir: &Path) -> Result<DirCliOutput, QianjiError> {
    if looks_like_workdir_dir(dir)? {
        let show = show_workdir(dir)?;
        return Ok(DirCliOutput {
            rendered: render_workdir_show(&show),
            exit_code: 0,
        });
    }

    if classify_flowhub_dir(dir)?.is_some() {
        let show = show_flowhub(dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_show(&show),
            exit_code: 0,
        });
    }

    if looks_like_flowhub_scenario_dir(dir) {
        let flowhub_root = resolve_default_flowhub_root().map_err(|error| {
            QianjiError::Topology(format!(
                "failed to resolve default Flowhub root for scenario `{}`: {error}",
                dir.display()
            ))
        })?;
        let show = show_flowhub_scenario(&flowhub_root, dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_scenario_show(&show),
            exit_code: 0,
        });
    }

    Err(QianjiError::Topology(format!(
        "`{}` is neither a bounded work surface, a Flowhub root/module, nor a Flowhub scenario directory",
        dir.display()
    )))
}

fn run_show_graph_command(graph: &Path) -> Result<DirCliOutput, QianjiError> {
    let show = show_flowhub_graph(graph)?;
    Ok(DirCliOutput {
        rendered: render_flowhub_graph_show(&show),
        exit_code: 0,
    })
}

fn run_show_contract_command(contract_name: &str) -> Result<DirCliOutput, QianjiError> {
    let show = show_wendao_docs_contract(contract_name)?;
    Ok(DirCliOutput {
        rendered: render_wendao_docs_contract_show(&show),
        exit_code: 0,
    })
}

fn run_show_anchored_scenario_command(
    anchor: &Path,
    scenario: &str,
    dir: Option<&Path>,
) -> Result<DirCliOutput, QianjiError> {
    Ok(DirCliOutput {
        rendered: show_flowhub_anchored_scenario(anchor, scenario, dir)?,
        exit_code: 0,
    })
}

fn run_check_dir_command(dir: &Path) -> Result<DirCliOutput, QianjiError> {
    if looks_like_workdir_dir(dir)? {
        let report = check_workdir(dir)?;
        return Ok(DirCliOutput {
            rendered: render_workdir_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if classify_flowhub_dir(dir)?.is_some() {
        let report = check_flowhub(dir)?;
        return Ok(DirCliOutput {
            rendered: render_flowhub_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if looks_like_flowhub_scenario_dir(dir) {
        let flowhub_root = resolve_default_flowhub_root().map_err(|error| {
            QianjiError::Topology(format!(
                "failed to resolve default Flowhub root for scenario `{}`: {error}",
                dir.display()
            ))
        })?;
        let report = check_flowhub_scenario(&flowhub_root, dir);
        return Ok(DirCliOutput {
            rendered: render_flowhub_scenario_check_markdown(&report),
            exit_code: if report.is_valid() { 0 } else { 2 },
        });
    }

    if !dir.exists() {
        return Ok(render_missing_workdir_root_output(dir));
    }

    if dir.is_dir() {
        return Ok(render_uninitialized_workdir_root_output(dir));
    }

    Err(QianjiError::Topology(format!(
        "`{}` is neither a bounded work surface, a Flowhub root/module, nor a Flowhub scenario directory",
        dir.display()
    )))
}

fn run_materialize_command(target: &MaterializeCliTarget) -> Result<DirCliOutput, QianjiError> {
    match target {
        MaterializeCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
            current_node,
        } => {
            let materialized = materialize_flowhub_anchored_scenario_at_node(
                anchor,
                scenario,
                dir,
                current_node.as_deref(),
            )?;
            Ok(DirCliOutput {
                rendered: render_anchored_materialized_workdir(&materialized),
                exit_code: 0,
            })
        }
    }
}

fn run_advance_command(dir: &Path, to: &str) -> Result<DirCliOutput, QianjiError> {
    let advance = advance_workdir_step(dir, to)?;
    Ok(DirCliOutput {
        rendered: render_workdir_advance(&advance),
        exit_code: 0,
    })
}

fn render_missing_workdir_root_output(dir: &Path) -> DirCliOutput {
    render_workdir_bootstrap_output(
        dir,
        "Missing workdir root",
        format!(
            "localized run root `{}` does not exist, so `qianji check` has no bounded work surface to evaluate",
            dir.display()
        ),
        format!(
            "create `{}` with `qianji.toml`, `flowchart.mmd`, and the scenario-declared runtime surfaces before rerunning `qianji check --dir {}`; use `qianji show --anchor <module-qianji.toml> --scenario <name> --dir {}` to inspect the expected surface first",
            dir.display(),
            dir.display(),
            dir.display()
        ),
    )
}

fn render_uninitialized_workdir_root_output(dir: &Path) -> DirCliOutput {
    render_workdir_bootstrap_output(
        dir,
        "Uninitialized workdir root",
        format!(
            "`{}` exists, but it does not yet declare a bounded work-surface manifest (`qianji.toml` with `[plan]` and `[check]`)",
            dir.display()
        ),
        format!(
            "initialize `{}` with `qianji.toml`, `flowchart.mmd`, and the scenario-declared runtime surfaces before rerunning `qianji check --dir {}`; use `qianji show --anchor <module-qianji.toml> --scenario <name> --dir {}` to inspect the expected surface first",
            dir.display(),
            dir.display(),
            dir.display()
        ),
    )
}

fn render_workdir_bootstrap_output(
    dir: &Path,
    title: &str,
    problem: String,
    fix: String,
) -> DirCliOutput {
    let report = WorkdirCheckReport {
        plan_name: "uninitialized-workdir".to_string(),
        workdir: dir.to_path_buf(),
        diagnostics: vec![WorkdirDiagnostic {
            title: title.to_string(),
            location: dir.to_path_buf(),
            problem,
            why_it_blocks:
                "Qianji cannot evaluate localized step boundaries until the run root is materialized"
                    .to_string(),
            fix,
            follow_up_surfaces: Vec::new(),
        }],
    };

    DirCliOutput {
        rendered: render_workdir_check_markdown(&report),
        exit_code: 2,
    }
}

fn resolve_default_flowhub_root() -> io::Result<PathBuf> {
    Ok(resolve_workspace_root(None)?.join("qianji-flowhub"))
}

async fn handle_contract_feedback_command(
    command: ContractFeedbackCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ContractFeedbackCliCommand::RestDocs(command) => {
            handle_rest_docs_contract_feedback(command).await
        }
    }
}

async fn handle_rest_docs_contract_feedback(
    command: RestDocsCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let openapi_path = resolve_cli_path(command.openapi_path.as_path())?;
    let workspace_root = resolve_workspace_root(command.workspace_root.as_deref())?;
    let mut collection_context =
        build_rest_docs_collection_context(&openapi_path, Some(workspace_root.clone()));
    collection_context.labels.insert(
        "invocation".to_string(),
        "qianji_contract_feedback_rest_docs".to_string(),
    );
    collection_context.labels.insert(
        "session_id".to_string(),
        build_contract_feedback_session_id(&openapi_path),
    );
    if let Some(model) = command.model.as_ref() {
        collection_context
            .labels
            .insert("llm_model".to_string(), model.clone());
    }

    let config = build_contract_feedback_config(&command);
    let advisory_roles = config
        .advisory_policy_for_pack(REST_DOCS_PACK_ID)
        .requested_roles;

    let output = if command.live_advisory {
        run_live_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    } else if advisory_roles.is_empty() {
        run_deterministic_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    } else {
        run_scaffold_rest_docs_contract_feedback(
            &command,
            &openapi_path,
            &workspace_root,
            collection_context,
            &config,
            advisory_roles,
        )
        .await?
    };

    print_contract_feedback_output(&output)
}

async fn run_live_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    let suite = build_rest_docs_contract_suite(openapi_path);
    let runtime = build_live_contract_feedback_runtime()?;
    let options = build_live_contract_feedback_options(command)?;

    if command.no_persist {
        let run = run_contract_feedback_flow_with_live_advisory(
            &suite,
            &collection_context,
            config,
            runtime.orchestrator,
            runtime.registry,
            runtime.client,
            options,
        )
        .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            true,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let persisted = run_and_persist_contract_feedback_flow_with_live_advisory(
        &suite,
        &collection_context,
        config,
        runtime,
        options,
        &sink,
    )
    .await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        true,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}

async fn run_deterministic_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    if command.no_persist {
        let run = run_rest_docs_contract_feedback(
            openapi_path,
            collection_context,
            config,
            &NoopAdvisoryAuditExecutor,
        )
        .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            false,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let persisted = run_and_persist_rest_docs_contract_feedback(
        openapi_path,
        collection_context,
        config,
        &sink,
    )
    .await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        false,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}

async fn run_scaffold_rest_docs_contract_feedback(
    command: &RestDocsCliCommand,
    openapi_path: &Path,
    workspace_root: &Path,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_roles: Vec<String>,
) -> Result<ContractFeedbackCliOutput, Box<dyn std::error::Error>> {
    let suite = build_rest_docs_contract_suite(openapi_path);
    let executor = build_scaffold_advisory_executor();

    if command.no_persist {
        let run =
            run_rest_docs_contract_feedback(openapi_path, collection_context, config, &executor)
                .await?;
        return Ok(build_contract_feedback_output(
            openapi_path.to_path_buf(),
            workspace_root.to_path_buf(),
            false,
            advisory_roles,
            run,
            Vec::new(),
            None,
        ));
    }

    let sink = build_contract_feedback_sink(command, workspace_root);
    let persisted = run_and_persist_contract_feedback_flow(
        &suite,
        &collection_context,
        config,
        &executor,
        &sink,
    )
    .await?;

    Ok(build_persisted_contract_feedback_output(
        openapi_path.to_path_buf(),
        workspace_root.to_path_buf(),
        false,
        advisory_roles,
        persisted,
        storage_output_from_sink(&sink),
    ))
}

fn print_contract_feedback_output(
    output: &ContractFeedbackCliOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn build_contract_feedback_output(
    openapi_path: PathBuf,
    workspace_root: PathBuf,
    live_advisory: bool,
    advisory_roles: Vec<String>,
    run: QianjiContractFeedbackRun,
    persisted_entry_ids: Vec<String>,
    storage: Option<ContractFeedbackStorageOutput>,
) -> ContractFeedbackCliOutput {
    ContractFeedbackCliOutput {
        openapi_path,
        workspace_root,
        live_advisory,
        advisory_roles,
        report: run.report,
        knowledge_entry_ids: run
            .knowledge_entries
            .into_iter()
            .map(|entry| entry.id)
            .collect(),
        persisted_entry_ids,
        storage,
    }
}

fn build_persisted_contract_feedback_output(
    openapi_path: impl Into<PathBuf>,
    workspace_root: impl Into<PathBuf>,
    live_advisory: bool,
    advisory_roles: Vec<String>,
    persisted: QianjiPersistedContractFeedbackRun,
    storage: ContractFeedbackStorageOutput,
) -> ContractFeedbackCliOutput {
    build_contract_feedback_output(
        openapi_path.into(),
        workspace_root.into(),
        live_advisory,
        advisory_roles,
        persisted.run,
        persisted.persisted_entry_ids,
        Some(storage),
    )
}

fn storage_output_from_sink(
    sink: &KnowledgeStorageContractFeedbackSink,
) -> ContractFeedbackStorageOutput {
    ContractFeedbackStorageOutput {
        storage_path: sink.storage_path().to_string(),
        table_name: sink.table_name().to_string(),
    }
}

fn build_contract_feedback_config(command: &RestDocsCliCommand) -> ContractRunConfig {
    let mut config = ContractRunConfig {
        generated_at: generated_at_string(),
        ..ContractRunConfig::default()
    };

    if command.live_advisory || !command.roles.is_empty() {
        config.set_advisory_policy_for_pack(
            REST_DOCS_PACK_ID,
            AdvisoryAuditPolicy {
                enabled: true,
                requested_roles: command.roles.clone(),
                min_severity: FindingSeverity::Warning,
            },
        );
    }

    config
}

fn build_scaffold_advisory_executor() -> QianjiAdvisoryAuditExecutor {
    let (orchestrator, registry) = build_contract_feedback_role_runtime();
    QianjiAdvisoryAuditExecutor::new(orchestrator, registry)
}

fn build_live_contract_feedback_runtime()
-> Result<QianjiLiveContractFeedbackRuntime, Box<dyn std::error::Error>> {
    let llm_runtime = resolve_qianji_runtime_llm_config()?;
    let (orchestrator, registry) = build_contract_feedback_role_runtime();
    let client: Arc<dyn LlmClient> = Arc::new(OpenAICompatibleClient {
        api_key: llm_runtime.api_key,
        base_url: llm_runtime.base_url,
        wire_api: OpenAIWireApi::parse(Some(llm_runtime.wire_api.as_str())),
        http: reqwest::Client::new(),
    });

    Ok(QianjiLiveContractFeedbackRuntime::new(
        orchestrator,
        registry,
        client,
    ))
}

fn build_contract_feedback_role_runtime() -> (Arc<ThousandFacesOrchestrator>, Arc<PersonaRegistry>)
{
    (
        Arc::new(ThousandFacesOrchestrator::new(
            "Contract Feedback".to_string(),
            None,
        )),
        Arc::new(PersonaRegistry::with_builtins()),
    )
}

fn build_live_contract_feedback_options(
    command: &RestDocsCliCommand,
) -> Result<QianjiLiveContractFeedbackOptions, Box<dyn std::error::Error>> {
    let mut options = QianjiLiveContractFeedbackOptions::default();
    let resolved = resolve_qianji_runtime_llm_config()?;
    options.model = command.model.clone().unwrap_or(resolved.model);
    if let Some(temperature) = command.temperature {
        options.temperature = temperature;
    }
    options.cognitive_early_halt_threshold = command.cognitive_early_halt_threshold;
    Ok(options)
}

fn build_contract_feedback_sink(
    command: &RestDocsCliCommand,
    workspace_root: &Path,
) -> KnowledgeStorageContractFeedbackSink {
    let storage_path = command
        .storage_path
        .clone()
        .unwrap_or_else(|| default_contract_feedback_storage_path(workspace_root));
    let storage_path = resolve_path_against_root(storage_path, workspace_root);

    KnowledgeStorageContractFeedbackSink::new(
        storage_path.display().to_string(),
        command.table_name.clone(),
    )
}

fn default_contract_feedback_storage_path(workspace_root: &Path) -> PathBuf {
    let resolved =
        resolve_cache_home(Some(workspace_root)).unwrap_or_else(|| workspace_root.join(".cache"));
    sanitize_prj_cache_home(workspace_root, resolved)
        .join("wendao")
        .join("contract_feedback")
}

fn sanitize_prj_cache_home(workspace_root: &Path, resolved: PathBuf) -> PathBuf {
    if resolved.is_absolute() && !resolved.starts_with(workspace_root) {
        workspace_root.join(".cache")
    } else {
        resolved
    }
}

fn build_contract_feedback_session_id(openapi_path: &Path) -> String {
    let raw = openapi_path.to_string_lossy();
    let mut normalized = String::with_capacity(raw.len());
    for character in raw.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push('_');
        }
    }
    format!("contract-feedback:rest-docs:{normalized}")
}

fn generated_at_string() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_error| "0".to_string(),
        |duration| duration.as_millis().to_string(),
    )
}

fn resolve_workspace_root(explicit: Option<&Path>) -> io::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let raw_project_root = env::var("PRJ_ROOT").ok();
    let base = explicit.map_or_else(
        || {
            resolve_project_root_or_cwd_from_value(
                raw_project_root.as_deref(),
                Some(current_dir.as_path()),
            )
        },
        Path::to_path_buf,
    );

    Ok(resolve_path_against_root(base, current_dir.as_path()))
}

fn resolve_cli_path(path: &Path) -> io::Result<PathBuf> {
    Ok(resolve_path_against_root(
        path.to_path_buf(),
        &env::current_dir()?,
    ))
}

fn resolve_path_against_root(path: PathBuf, root: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn parse_contract_feedback_command(
    args: &[String],
) -> io::Result<Option<ContractFeedbackCliCommand>> {
    if args.get(1).map(String::as_str) != Some("contract-feedback") {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("rest-docs") => Ok(Some(ContractFeedbackCliCommand::RestDocs(
            parse_rest_docs_cli_command(&args[3..])?,
        ))),
        Some(other) => Err(invalid_input(format!(
            "unsupported contract-feedback subcommand '{other}'"
        ))),
        None => Err(invalid_input(
            "missing contract-feedback subcommand; expected 'rest-docs'",
        )),
    }
}

fn parse_rest_docs_cli_command(args: &[String]) -> io::Result<RestDocsCliCommand> {
    let Some(openapi_path) = args.first() else {
        return Err(invalid_input(
            "missing OpenAPI path; expected 'contract-feedback rest-docs <openapi_path>'",
        ));
    };

    let mut command = RestDocsCliCommand {
        openapi_path: PathBuf::from(openapi_path),
        workspace_root: None,
        storage_path: None,
        table_name: DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME.to_string(),
        no_persist: false,
        live_advisory: false,
        roles: Vec::new(),
        model: None,
        temperature: None,
        cognitive_early_halt_threshold: None,
    };

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--workspace-root" => {
                command.workspace_root = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--workspace-root",
                )?));
            }
            "--storage-path" => {
                command.storage_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--storage-path",
                )?));
            }
            "--table-name" => {
                command.table_name = parse_flag_value(args, &mut index, "--table-name")?;
            }
            "--role" => {
                command
                    .roles
                    .push(parse_flag_value(args, &mut index, "--role")?);
            }
            "--model" => {
                command.model = Some(parse_flag_value(args, &mut index, "--model")?);
            }
            "--temperature" => {
                let raw = parse_flag_value(args, &mut index, "--temperature")?;
                command.temperature = Some(raw.parse::<f32>().map_err(|error| {
                    invalid_input(format!(
                        "failed to parse --temperature value '{raw}' as f32: {error}"
                    ))
                })?);
            }
            "--cognitive-threshold" => {
                let raw = parse_flag_value(args, &mut index, "--cognitive-threshold")?;
                command.cognitive_early_halt_threshold =
                    Some(raw.parse::<f32>().map_err(|error| {
                        invalid_input(format!(
                            "failed to parse --cognitive-threshold value '{raw}' as f32: {error}"
                        ))
                    })?);
            }
            "--no-persist" => {
                command.no_persist = true;
            }
            "--live-advisory" => {
                command.live_advisory = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported contract-feedback option '{other}'"
                )));
            }
        }

        index += 1;
    }

    Ok(command)
}

fn parse_dir_command(args: &[String]) -> io::Result<Option<DirCliCommand>> {
    match args.get(1).map(String::as_str) {
        Some("show") => Ok(Some(DirCliCommand::Show {
            target: parse_show_target(&args[2..])?,
        })),
        Some("check") => Ok(Some(DirCliCommand::Check {
            dir: parse_dir_flag(&args[2..], "check")?,
        })),
        Some("materialize") => Ok(Some(DirCliCommand::Materialize {
            target: parse_materialize_target(&args[2..])?,
        })),
        Some("advance") => {
            let (dir, to) = parse_advance_command(&args[2..])?;
            Ok(Some(DirCliCommand::Advance { dir, to }))
        }
        _ => Ok(None),
    }
}

fn parse_show_target(args: &[String]) -> io::Result<ShowCliTarget> {
    let mut index = 0;
    let mut dir = None;
    let mut graph = None;
    let mut contract = None;
    let mut anchor = None;
    let mut scenario = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --dir in `show` command"))?;
                dir = Some(PathBuf::from(value));
            }
            "--graph" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --graph in `show` command"))?;
                graph = Some(PathBuf::from(value));
            }
            "--contract" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --contract in `show` command")
                })?;
                contract = Some(value.clone());
            }
            "--anchor" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --anchor in `show` command"))?;
                anchor = Some(PathBuf::from(value));
            }
            "--scenario" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --scenario in `show` command")
                })?;
                scenario = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `show` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (anchor, scenario, dir, graph, contract) {
        (Some(anchor), Some(scenario), dir, None, None) => Ok(ShowCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
        }),
        (Some(_), None, _, None, None) | (None, Some(_), _, None, None) => Err(invalid_input(
            "`show --anchor` requires `--scenario <ref>` and `show --scenario` requires `--anchor <path>`",
        )),
        (None, None, Some(dir), None, None) => Ok(ShowCliTarget::Dir(dir)),
        (None, None, None, Some(graph), None) => Ok(ShowCliTarget::Graph(graph)),
        (None, None, None, None, Some(contract_name)) => Ok(ShowCliTarget::Contract(contract_name)),
        (None, None, None, None, None) => Err(invalid_input(
            "missing `--dir <path>`, `--graph <path>`, `--contract <name>`, or `--anchor <path> --scenario <ref>` for `show` command",
        )),
        _ => Err(invalid_input(
            "`show` command requires exactly one of `--dir <path>`, `--graph <path>`, `--contract <name>`, or `--anchor <path> --scenario <ref>`",
        )),
    }
}

fn parse_dir_flag(args: &[String], command: &str) -> io::Result<PathBuf> {
    let mut index = 0;
    let mut dir = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input(format!("missing value for --dir in `{command}` command"))
                })?;
                dir = Some(PathBuf::from(value));
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `{command}` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    dir.ok_or_else(|| invalid_input(format!("missing `--dir <path>` for `{command}` command")))
}

fn parse_materialize_target(args: &[String]) -> io::Result<MaterializeCliTarget> {
    let mut index = 0;
    let mut anchor = None;
    let mut scenario = None;
    let mut dir = None;
    let mut current_node = None;
    while index < args.len() {
        match args[index].as_str() {
            "--anchor" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --anchor in `materialize` command")
                })?;
                anchor = Some(PathBuf::from(value));
            }
            "--scenario" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --scenario in `materialize` command")
                })?;
                scenario = Some(value.clone());
            }
            "--dir" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --dir in `materialize` command")
                })?;
                dir = Some(PathBuf::from(value));
            }
            "--current-node" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --current-node in `materialize` command")
                })?;
                current_node = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `materialize` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (anchor, scenario, dir) {
        (Some(anchor), Some(scenario), Some(dir)) => Ok(MaterializeCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
            current_node,
        }),
        _ => Err(invalid_input(
            "missing `--anchor <path> --scenario <ref> --dir <path>` for `materialize` command",
        )),
    }
}

fn parse_advance_command(args: &[String]) -> io::Result<(PathBuf, String)> {
    let mut index = 0;
    let mut dir = None;
    let mut to = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --dir in `advance` command"))?;
                dir = Some(PathBuf::from(value));
            }
            "--to" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --to in `advance` command"))?;
                to = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `advance` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (dir, to) {
        (Some(dir), Some(to)) => Ok((dir, to)),
        _ => Err(invalid_input(
            "missing `--dir <path> --to <node>` for `advance` command",
        )),
    }
}

fn parse_flag_value(args: &[String], index: &mut usize, flag: &str) -> io::Result<String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| invalid_input(format!("missing value for {flag}")))
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn print_qianji_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  Execution: qianji [-v|--log-verbose] <repo_path> <manifest_path> <context_json> [session_id]"
    );
    eprintln!("  Graph:     qianji [-v|--log-verbose] graph <manifest_path> <output_path>");
    eprintln!("  Show:      qianji [-v|--log-verbose] show --dir <path>");
    eprintln!("             qianji [-v|--log-verbose] show --graph <path>");
    eprintln!("             qianji [-v|--log-verbose] show --contract <id>");
    eprintln!(
        "  Materialize: qianji [-v|--log-verbose] materialize --anchor <path> --scenario <ref> --dir <path> [--current-node <node>]"
    );
    eprintln!("  Advance:   qianji [-v|--log-verbose] advance --dir <path> --to <node>");
    eprintln!("  Check:     qianji [-v|--log-verbose] check --dir <path>");
    eprintln!(
        "  Contract:  qianji [-v|--log-verbose] contract-feedback rest-docs <openapi_path> [--workspace-root PATH] [--storage-path PATH] [--table-name NAME] [--role ROLE]... [--no-persist] [--live-advisory] [--model MODEL] [--temperature FLOAT] [--cognitive-threshold FLOAT]"
    );
}

fn handle_graph_export(
    manifest_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Qianji Graph from: {manifest_path}");

    let manifest_toml = fs::read_to_string(manifest_path)?;

    // Using simple defaults for the compiler as we only need the topology
    let index = Arc::new(LinkGraphIndex::build(std::env::temp_dir().as_path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Visualizer".into(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    // Provide a dummy client to satisfy compilation check for LLM nodes
    let llm_client: Option<Arc<QianjiLlmClient>> = Some(Arc::new(NoopLlmClient));

    let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
    let engine = compiler.compile(&manifest_toml)?;

    let layout_engine = QianjiLayoutEngine::new(QgsTheme::default());
    let layout_result = layout_engine.compute_from_engine(&engine);
    let bpmn_xml = generate_bpmn_xml(&layout_result);

    // Export rich knowledge graph for 3D view
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

#[cfg(test)]
#[path = "../../tests/unit/bin/qianji/mod.rs"]
mod tests;
