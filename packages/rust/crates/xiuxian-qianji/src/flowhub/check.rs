use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use globset::Glob;
use walkdir::WalkDir;

use crate::contracts::{
    FlowhubGraphContract, FlowhubStructureContract, FlowhubValidationKind, FlowhubValidationRule,
    FlowhubValidationScope,
};
use crate::error::QianjiError;
use crate::flowhub::{
    analyze_mermaid_flowchart_topology, parse_mermaid_flowchart, validate_mermaid_flowchart,
};
use crate::markdown::{MarkdownDiagnostic, render_validation_failed, render_validation_pass};
use crate::{ResolvedFlowhubModule, resolve_flowhub_module_children};

use super::discover::{
    FlowhubDirKind, FlowhubDiscoveredModule, FlowhubModuleCandidate, classify_flowhub_dir,
    discover_all_flowhub_module_refs, find_flowhub_root_for_module_dir,
    load_flowhub_module_candidate, module_candidate_from_dir, module_candidate_from_ref,
};
use super::load::load_flowhub_root_manifest;
use super::{
    FlowhubGraphAnnotations, FlowhubScenarioIr, compile_flowhub_scenario_ir,
    parse_flowhub_graph_annotations, resolve_flowhub_graph_name,
};

/// One user-facing validation diagnostic for a Flowhub root or module check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubDiagnostic {
    /// Short diagnostic title.
    pub title: String,
    /// On-disk location of the failing surface.
    pub location: PathBuf,
    /// Concrete failing condition.
    pub problem: String,
    /// Why the issue blocks continued Flowhub use.
    pub why_it_blocks: String,
    /// Concrete next action for repairing the failing surface.
    pub fix: String,
}

/// Structural validation result for a Flowhub root or single module target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubCheckReport {
    /// Checked Flowhub root or module path.
    pub target: PathBuf,
    /// Count of modules that were traversed during validation.
    pub checked_modules: usize,
    /// Collected blocking diagnostics.
    pub diagnostics: Vec<FlowhubDiagnostic>,
}

struct MermaidCaseValidation<'a> {
    module_ref: &'a str,
    scenario_case: &'a Path,
    file_name: &'a str,
    merimind_graph_name: &'a str,
    declared_graph: Option<&'a FlowhubGraphContract>,
    annotations: Option<&'a FlowhubGraphAnnotations>,
}

impl FlowhubCheckReport {
    /// Returns `true` when no blocking diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Validate a Flowhub library root or a single Flowhub module directory.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the target is not Flowhub-shaped or
/// its filesystem cannot be traversed.
pub fn check_flowhub(dir: impl AsRef<Path>) -> Result<FlowhubCheckReport, QianjiError> {
    let dir = dir.as_ref();
    match classify_flowhub_dir(dir)? {
        Some(FlowhubDirKind::Root) => check_flowhub_root(dir),
        Some(FlowhubDirKind::Module) => check_flowhub_module(dir),
        None => Err(QianjiError::Topology(format!(
            "`{}` is not a Flowhub root or module directory",
            dir.display()
        ))),
    }
}

/// Render a Flowhub validation report into markdown diagnostics.
#[must_use]
pub fn render_flowhub_check_markdown(report: &FlowhubCheckReport) -> String {
    if report.is_valid() {
        return render_validation_pass(&[
            format!("Location: {}", report.target.display()),
            format!("Checked modules: {}", report.checked_modules),
        ]);
    }

    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| MarkdownDiagnostic {
            title: diagnostic.title.as_str(),
            location: diagnostic.location.display().to_string().into(),
            problem: diagnostic.problem.as_str(),
            why_it_blocks: diagnostic.why_it_blocks.as_str(),
            fix: diagnostic.fix.as_str(),
        })
        .collect::<Vec<_>>();

    render_validation_failed(
        &[
            format!("Location: {}", report.target.display()),
            format!("Checked modules: {}", report.checked_modules),
        ],
        &diagnostics,
    )
}

fn check_flowhub_root(root: &Path) -> Result<FlowhubCheckReport, QianjiError> {
    let mut diagnostics = Vec::new();
    let root_manifest = match load_flowhub_root_manifest(root.join("qianji.toml")) {
        Ok(manifest) => Some(manifest),
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid Flowhub root contract".to_string(),
                location: root.join("qianji.toml"),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the root-level graph-module structure contract"
                    .to_string(),
                fix: "create or repair the Flowhub root `qianji.toml` so it defines `[contract]`"
                    .to_string(),
            });
            None
        }
    };

    let Some(root_manifest) = root_manifest else {
        return Ok(FlowhubCheckReport {
            target: root.to_path_buf(),
            checked_modules: 0,
            diagnostics,
        });
    };

    validate_root_contract(root, &root_manifest.contract, &mut diagnostics)?;
    let known_module_names = discover_all_flowhub_module_refs(root)?;

    let candidates = root_manifest
        .contract
        .register
        .iter()
        .map(|module_ref| module_candidate_from_ref(root, module_ref))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        diagnostics.push(FlowhubDiagnostic {
            title: "No Flowhub modules".to_string(),
            location: root.to_path_buf(),
            problem: "the Flowhub root contract does not register any top-level graph modules"
                .to_string(),
            why_it_blocks: "Qianji cannot expose or validate any reusable Flowhub graph nodes"
                .to_string(),
            fix: "add at least one `contract.register` entry in the Flowhub root manifest"
                .to_string(),
        });
        return Ok(FlowhubCheckReport {
            target: root.to_path_buf(),
            checked_modules: 0,
            diagnostics,
        });
    }

    validate_unregistered_top_level_directories(root, &root_manifest.contract, &mut diagnostics)?;

    let mut checked_modules = 0;
    let mut visited = BTreeSet::new();
    for candidate in &candidates {
        if !candidate.manifest_path.is_file() {
            continue;
        }
        checked_modules += validate_candidate(
            candidate,
            &known_module_names,
            &mut diagnostics,
            &mut visited,
        )?;
    }

    Ok(FlowhubCheckReport {
        target: root.to_path_buf(),
        checked_modules,
        diagnostics,
    })
}

fn check_flowhub_module(module_dir: &Path) -> Result<FlowhubCheckReport, QianjiError> {
    let mut diagnostics = Vec::new();
    let candidate = module_candidate_from_dir(module_dir)?;
    let known_module_names = load_known_module_names_for_module(module_dir);
    let mut visited = BTreeSet::new();
    let checked_modules = validate_candidate(
        &candidate,
        &known_module_names,
        &mut diagnostics,
        &mut visited,
    )?;

    Ok(FlowhubCheckReport {
        target: module_dir.to_path_buf(),
        checked_modules,
        diagnostics,
    })
}

fn validate_candidate(
    candidate: &FlowhubModuleCandidate,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
    visited: &mut BTreeSet<String>,
) -> Result<usize, QianjiError> {
    let module = match load_flowhub_module_candidate(candidate) {
        Ok(module) => module,
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid Flowhub module manifest".to_string(),
                location: candidate.manifest_path.clone(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the module contract or exported handles"
                    .to_string(),
                fix: "repair the module-root `qianji.toml` so it satisfies the Flowhub contract"
                    .to_string(),
            });
            return Ok(1);
        }
    };

    validate_loaded_module(&module, known_module_names, diagnostics, visited)
}

fn validate_loaded_module(
    module: &FlowhubDiscoveredModule,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
    visited: &mut BTreeSet<String>,
) -> Result<usize, QianjiError> {
    if !visited.insert(module.module_ref.clone()) {
        return Ok(0);
    }

    let mut checked_modules = 1;
    let expected_module_name = last_module_segment(&module.module_ref);
    if module.manifest.module.name != expected_module_name {
        diagnostics.push(FlowhubDiagnostic {
            title: "Mismatched module name".to_string(),
            location: module.manifest_path.clone(),
            problem: format!(
                "module reference `{}` ends with `{expected_module_name}`, but `module.name = \"{}\"`",
                module.module_ref, module.manifest.module.name
            ),
            why_it_blocks: "the filesystem path and declared module identity diverge".to_string(),
            fix: format!(
                "rename `module.name` to `{expected_module_name}` or move the module directory"
            ),
        });
    }

    for rule in module
        .manifest
        .validation
        .iter()
        .filter(|rule| rule.scope == FlowhubValidationScope::Module)
    {
        validate_module_rule(module, rule, diagnostics);
    }

    if let Some(contract) = &module.manifest.contract {
        validate_registered_contract(module, contract, diagnostics)?;
        validate_mermaid_case_files(module, Some(contract), known_module_names, diagnostics)?;
    } else {
        validate_unregistered_child_directories(module, None, diagnostics)?;
        validate_mermaid_case_files(module, None, known_module_names, diagnostics)?;
    }

    if module.manifest.template.is_some() {
        let resolved_parent = ResolvedFlowhubModule {
            alias: module.manifest.module.name.clone(),
            module_ref: module.module_ref.clone(),
            module_name: module.manifest.module.name.clone(),
            module_dir: module.module_dir.clone(),
            manifest_path: module.manifest_path.clone(),
            manifest: module.manifest.clone(),
        };

        match resolve_flowhub_module_children(&resolved_parent) {
            Ok(children) => {
                for child in &children {
                    checked_modules +=
                        validate_resolved_module(child, known_module_names, diagnostics, visited)?;
                }
            }
            Err(error) => diagnostics.push(FlowhubDiagnostic {
                title: "Unresolved composite child".to_string(),
                location: module.manifest_path.clone(),
                problem: error.to_string(),
                why_it_blocks: "the composite Flowhub module cannot assemble its internal graph"
                    .to_string(),
                fix: "repair `template.use` so every declared child module resolves".to_string(),
            }),
        }
    }

    Ok(checked_modules)
}

fn validate_resolved_module(
    module: &ResolvedFlowhubModule,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
    visited: &mut BTreeSet<String>,
) -> Result<usize, QianjiError> {
    let discovered = FlowhubDiscoveredModule {
        module_ref: module.module_ref.clone(),
        module_dir: module.module_dir.clone(),
        manifest_path: module.manifest_path.clone(),
        manifest: module.manifest.clone(),
    };
    validate_loaded_module(&discovered, known_module_names, diagnostics, visited)
}

fn validate_module_rule(
    module: &FlowhubDiscoveredModule,
    rule: &FlowhubValidationRule,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) {
    match rule.kind {
        FlowhubValidationKind::Dir => {
            let path = module.module_dir.join(&rule.path);
            if !path.exists() {
                if rule.required {
                    diagnostics.push(missing_path_diagnostic(module, &path, rule, "directory"));
                }
                return;
            }
            if !path.is_dir() {
                diagnostics.push(type_mismatch_diagnostic(module, &path, rule, "directory"));
            }
        }
        FlowhubValidationKind::File => {
            let path = module.module_dir.join(&rule.path);
            if !path.exists() {
                if rule.required {
                    diagnostics.push(missing_path_diagnostic(module, &path, rule, "file"));
                }
                return;
            }
            if !path.is_file() {
                diagnostics.push(type_mismatch_diagnostic(module, &path, rule, "file"));
            }
        }
        FlowhubValidationKind::Glob => {
            let min_matches = rule.min_matches.unwrap_or(1);
            match count_glob_matches(&module.module_dir, &rule.path) {
                Ok(matches) if matches < min_matches => diagnostics.push(FlowhubDiagnostic {
                    title: "Missing module glob matches".to_string(),
                    location: module.module_dir.clone(),
                    problem: format!(
                        "module `{}` requires at least {min_matches} match(es) for `{}`, but found {matches}",
                        module.module_ref, rule.path
                    ),
                    why_it_blocks: "the declared module surface is structurally incomplete"
                        .to_string(),
                    fix: format!(
                        "add files matching `{}` under `{}` or relax the module validation rule",
                        rule.path,
                        module.module_dir.display()
                    ),
                }),
                Ok(_) => {}
                Err(error) => diagnostics.push(FlowhubDiagnostic {
                    title: "Invalid module glob rule".to_string(),
                    location: module.manifest_path.clone(),
                    problem: error.to_string(),
                    why_it_blocks: "Qianji cannot evaluate the declared module validation rule"
                        .to_string(),
                    fix: format!("repair the glob pattern `{}` in `[[validation]]`", rule.path),
                }),
            }
        }
    }
}

fn validate_root_contract(
    root: &Path,
    contract: &FlowhubStructureContract,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    validate_contract_required_entries(
        root,
        contract,
        diagnostics,
        "Flowhub root",
        root,
        "root graph-module structure no longer matches the declared contract",
    )?;
    validate_unregistered_top_level_directories(root, contract, diagnostics)
}

fn validate_registered_contract(
    module: &FlowhubDiscoveredModule,
    contract: &FlowhubStructureContract,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    validate_contract_required_entries(
        &module.module_dir,
        contract,
        diagnostics,
        &format!("module `{}`", module.module_ref),
        &module.manifest_path,
        "the module contract no longer matches the on-disk Flowhub structure",
    )?;
    validate_unregistered_child_directories(module, Some(contract), diagnostics)
}

fn validate_contract_required_entries(
    base_dir: &Path,
    contract: &FlowhubStructureContract,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
    owner_label: &str,
    owner_location: &Path,
    why_it_blocks: &str,
) -> Result<(), QianjiError> {
    for requirement in expanded_required_entries(contract) {
        if is_glob_pattern(&requirement) {
            let matches = count_root_glob_matches(base_dir, &requirement)?;
            if matches == 0 {
                diagnostics.push(FlowhubDiagnostic {
                    title: "Missing contract glob matches".to_string(),
                    location: owner_location.to_path_buf(),
                    problem: format!(
                        "{owner_label} contract requires at least one match for `{requirement}`, but none were found"
                    ),
                    why_it_blocks: why_it_blocks.to_string(),
                    fix: format!(
                        "create a path matching `{requirement}` or relax `contract.required`"
                    ),
                });
            }
            continue;
        }

        let path = base_dir.join(&requirement);
        if !path.exists() {
            diagnostics.push(FlowhubDiagnostic {
                title: "Missing contract path".to_string(),
                location: path,
                problem: format!(
                    "{owner_label} contract requires `{requirement}`, but the path is absent"
                ),
                why_it_blocks: why_it_blocks.to_string(),
                fix: format!("create `{requirement}` or relax `contract.required`"),
            });
        }
    }

    Ok(())
}

fn expanded_required_entries(contract: &FlowhubStructureContract) -> Vec<String> {
    let mut entries = Vec::new();
    for requirement in &contract.required {
        if let Some(suffix) = requirement.strip_prefix("*/") {
            for child in &contract.register {
                entries.push(format!("{child}/{suffix}"));
            }
            continue;
        }
        entries.push(requirement.clone());
    }
    entries
}

fn validate_unregistered_top_level_directories(
    root: &Path,
    contract: &FlowhubStructureContract,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let allowed = allowed_immediate_child_directories(Some(contract));
    for child_dir in discover_immediate_child_directories(root)? {
        if allowed.contains(child_dir.as_str()) {
            continue;
        }
        diagnostics.push(FlowhubDiagnostic {
            title: "Unregistered Flowhub module".to_string(),
            location: root.join(&child_dir),
            problem: format!(
                "top-level directory `{child_dir}` exists on disk but is not declared in `contract.register` and is not implied by `contract.required`",
            ),
            why_it_blocks: "the Flowhub root graph has drifted away from its declared contract"
                .to_string(),
            fix: format!(
                "add `{child_dir}` to `contract.register` and `contract.required`, or remove the unregistered top-level directory",
            ),
        });
    }
    Ok(())
}

fn validate_unregistered_child_directories(
    module: &FlowhubDiscoveredModule,
    contract: Option<&FlowhubStructureContract>,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let allowed = allowed_immediate_child_directories(contract);
    for child_dir in discover_immediate_child_directories(&module.module_dir)? {
        if allowed.contains(child_dir.as_str()) {
            continue;
        }
        diagnostics.push(FlowhubDiagnostic {
            title: "Unregistered child directory".to_string(),
            location: module.module_dir.join(&child_dir),
            problem: format!(
                "module `{}` contains child directory `{child_dir}`, but it is not declared in `contract.register` and is not implied by `contract.required`",
                module.module_ref
            ),
            why_it_blocks: "the module graph shape has drifted away from its declared contract"
                .to_string(),
            fix: format!(
                "add `{child_dir}` to `contract.register` and `contract.required`, or remove the unregistered child directory"
            ),
        });
    }
    Ok(())
}

fn validate_mermaid_case_files(
    module: &FlowhubDiscoveredModule,
    contract: Option<&FlowhubStructureContract>,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    for scenario_case in discover_immediate_mermaid_files(&module.module_dir)? {
        let Some(file_name) = scenario_case.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !mermaid_file_is_contracted(file_name, contract) {
            diagnostics.push(FlowhubDiagnostic {
                title: "Uncontracted scenario-case graph".to_string(),
                location: scenario_case.clone(),
                problem: format!(
                    "module `{}` contains Mermaid scenario-case `{file_name}`, but the file is not declared by `contract.required`",
                    module.module_ref
                ),
                why_it_blocks: "scenario-case graphs must be owned by the node contract"
                    .to_string(),
                fix: format!(
                    "add `{file_name}` to `contract.required` or remove the uncontracted Mermaid file"
                ),
            });
            continue;
        }

        validate_contracted_mermaid_case(
            module,
            &scenario_case,
            file_name,
            known_module_names,
            diagnostics,
        )?;
    }

    Ok(())
}

fn validate_contracted_mermaid_case(
    module: &FlowhubDiscoveredModule,
    scenario_case: &Path,
    file_name: &str,
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Result<(), QianjiError> {
    let source = std::fs::read_to_string(scenario_case).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Mermaid scenario-case `{}`: {error}",
            scenario_case.display()
        ))
    })?;
    let fallback_graph_name = scenario_case
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(file_name);
    let declared_graph = declared_graph_contract(module, file_name);
    let annotations = match parse_flowhub_graph_annotations(&source) {
        Ok(annotations) => annotations,
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case contract".to_string(),
                location: scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the Mermaid-owned graph contract metadata"
                    .to_string(),
                fix: "repair the `%% qianji.*` annotations so the graph contract is well formed"
                    .to_string(),
            });
            return Ok(());
        }
    };
    let merimind_graph_name =
        resolve_flowhub_graph_name(annotations.as_ref(), declared_graph, fallback_graph_name);
    match parse_mermaid_flowchart(&source, &merimind_graph_name, known_module_names) {
        Ok(flowchart) => {
            validate_parsed_mermaid_case(
                &flowchart,
                &MermaidCaseValidation {
                    module_ref: &module.module_ref,
                    scenario_case,
                    file_name,
                    merimind_graph_name: &merimind_graph_name,
                    declared_graph,
                    annotations: annotations.as_ref(),
                },
                diagnostics,
            );
            Ok(())
        }
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case Mermaid".to_string(),
                location: scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot parse the scenario-case graph into nodes and edges"
                    .to_string(),
                fix: "repair the Mermaid flowchart syntax so node ids, labels, and edges are well formed".to_string(),
            });
            Ok(())
        }
    }
}

fn validate_parsed_mermaid_case(
    flowchart: &crate::flowhub::MermaidFlowchart,
    validation: &MermaidCaseValidation<'_>,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) {
    let scenario_ir = match compile_flowhub_scenario_ir(
        validation.scenario_case,
        validation.merimind_graph_name,
        flowchart,
        validation.annotations,
        validation.declared_graph,
    ) {
        Ok(contract) => contract,
        Err(error) => {
            diagnostics.push(FlowhubDiagnostic {
                title: "Invalid scenario-case contract".to_string(),
                location: validation.scenario_case.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot trust the compiled scenario-case contract surface"
                    .to_string(),
                fix: "repair the Mermaid annotations or legacy graph contract so the scenario surface compiles".to_string(),
            });
            return;
        }
    };
    let allowed_graph_node_labels = scenario_ir
        .as_ref()
        .map_or_else(BTreeSet::new, FlowhubScenarioIr::allowed_graph_node_labels);
    if let Err(problem) = validate_mermaid_flowchart(flowchart, &allowed_graph_node_labels) {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid scenario-case graph".to_string(),
            location: validation.scenario_case.to_path_buf(),
            problem,
            why_it_blocks:
                "Qianji cannot trust the scenario-case graph as a valid modular assembly surface"
                    .to_string(),
            fix: "repair the Mermaid node and edge graph so required module nodes are valid"
                .to_string(),
        });
        return;
    }

    let topology = analyze_mermaid_flowchart_topology(flowchart);
    if let Some(declared_topology) = scenario_ir
        .as_ref()
        .and_then(|graph| graph.declared_topology)
        && topology.topology != declared_topology
    {
        diagnostics.push(FlowhubDiagnostic {
            title: "Invalid scenario-case topology".to_string(),
            location: validation.scenario_case.to_path_buf(),
            problem: format!(
                "module `{}` expects scenario-case `{}` to resolve topology `{}`, but petgraph analysis resolved `{}`",
                validation.module_ref,
                validation.file_name,
                declared_topology.as_str(),
                topology.topology.as_str(),
            ),
            why_it_blocks:
                "Qianji cannot trust the scenario-case graph as a correctly typed Flowhub topology surface"
                    .to_string(),
            fix: format!(
                "repair `{}` so it matches `{}`, or update the owning graph contract to the analyzed graph shape",
                validation.file_name,
                declared_topology.as_str(),
            ),
        });
    }
}

fn declared_graph_contract<'a>(
    module: &'a FlowhubDiscoveredModule,
    file_name: &str,
) -> Option<&'a FlowhubGraphContract> {
    module
        .manifest
        .graph
        .iter()
        .find(|graph| graph.path == file_name)
}

fn discover_immediate_child_directories(module_dir: &Path) -> Result<Vec<String>, QianjiError> {
    let mut child_dirs = Vec::new();
    for entry in std::fs::read_dir(module_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module directory `{}`: {error}",
            module_dir.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module entry under `{}`: {error}",
                module_dir.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        child_dirs.push(name.to_string());
    }
    child_dirs.sort();
    Ok(child_dirs)
}

fn discover_immediate_mermaid_files(module_dir: &Path) -> Result<Vec<PathBuf>, QianjiError> {
    let mut mermaid_files = Vec::new();
    for entry in std::fs::read_dir(module_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module directory `{}`: {error}",
            module_dir.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module entry under `{}`: {error}",
                module_dir.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("mmd") {
            continue;
        }
        mermaid_files.push(path);
    }
    mermaid_files.sort();
    Ok(mermaid_files)
}

fn allowed_immediate_child_directories(
    contract: Option<&FlowhubStructureContract>,
) -> BTreeSet<String> {
    let mut allowed = BTreeSet::new();
    let Some(contract) = contract else {
        return allowed;
    };

    for entry in &contract.register {
        if let Some(first_segment) = entry.split('/').next() {
            allowed.insert(first_segment.to_string());
        }
    }

    for entry in &contract.required {
        let Some(first_segment) = entry.split('/').next() else {
            continue;
        };
        if first_segment == "*" || is_glob_pattern(first_segment) {
            continue;
        }
        allowed.insert(first_segment.to_string());
    }

    allowed
}

fn mermaid_file_is_contracted(
    file_name: &str,
    contract: Option<&FlowhubStructureContract>,
) -> bool {
    let Some(contract) = contract else {
        return false;
    };

    expanded_required_entries(contract).iter().any(|entry| {
        if entry == file_name {
            return true;
        }
        if is_glob_pattern(entry) {
            return Glob::new(entry).is_ok_and(|glob| glob.compile_matcher().is_match(file_name));
        }
        false
    })
}

fn load_known_module_names_for_module(module_dir: &Path) -> Vec<String> {
    let Ok(flowhub_root) = find_flowhub_root_for_module_dir(module_dir) else {
        return Vec::new();
    };

    discover_all_flowhub_module_refs(&flowhub_root).unwrap_or_default()
}

fn missing_path_diagnostic(
    module: &FlowhubDiscoveredModule,
    path: &Path,
    rule: &FlowhubValidationRule,
    expected_kind: &str,
) -> FlowhubDiagnostic {
    FlowhubDiagnostic {
        title: format!("Missing module {expected_kind}"),
        location: path.to_path_buf(),
        problem: format!(
            "module `{}` requires `{}` as a {expected_kind}, but the path is absent",
            module.module_ref, rule.path
        ),
        why_it_blocks: "the module contract no longer matches the on-disk Flowhub structure"
            .to_string(),
        fix: format!("create `{}` or relax the module validation rule", rule.path),
    }
}

fn type_mismatch_diagnostic(
    module: &FlowhubDiscoveredModule,
    path: &Path,
    rule: &FlowhubValidationRule,
    expected_kind: &str,
) -> FlowhubDiagnostic {
    FlowhubDiagnostic {
        title: format!("Invalid module {expected_kind}"),
        location: path.to_path_buf(),
        problem: format!(
            "module `{}` requires `{}` to be a {expected_kind}, but the existing path has a different type",
            module.module_ref, rule.path
        ),
        why_it_blocks: "the module contract and actual filesystem surface have diverged"
            .to_string(),
        fix: format!("repair `{}` so it is a {expected_kind}", rule.path),
    }
}

fn count_glob_matches(module_dir: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid Flowhub module glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    let mut match_count = 0_usize;
    for entry in WalkDir::new(module_dir) {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?;
        if entry.path() == module_dir {
            continue;
        }
        let relative = entry.path().strip_prefix(module_dir).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize Flowhub module path `{}` against `{}`: {error}",
                entry.path().display(),
                module_dir.display()
            ))
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if matcher.is_match(normalized.as_str()) {
            match_count += 1;
        }
    }
    Ok(match_count)
}

fn last_module_segment(module_ref: &str) -> &str {
    module_ref.rsplit('/').next().unwrap_or(module_ref)
}

fn count_root_glob_matches(root: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid Flowhub contract glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    let mut match_count = 0_usize;
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk Flowhub root `{}`: {error}",
                root.display()
            ))
        })?;
        if entry.path() == root {
            continue;
        }
        let relative = entry.path().strip_prefix(root).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize Flowhub root path `{}` against `{}`: {error}",
                entry.path().display(),
                root.display()
            ))
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if matcher.is_match(normalized.as_str()) {
            match_count += 1;
        }
    }

    Ok(match_count)
}

fn is_glob_pattern(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}
