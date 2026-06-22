use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::{FlowhubValidationKind, FlowhubValidationRule, FlowhubValidationScope};
use crate::error::QianjiError;
use crate::flowhub::discover::{
    FlowhubDiscoveredModule, FlowhubModuleCandidate, discover_all_flowhub_module_refs,
    find_flowhub_root_for_module_dir, load_flowhub_module_candidate, module_candidate_from_dir,
    module_candidate_from_ref,
};
use crate::flowhub::load::load_flowhub_root_manifest;
use crate::{ResolvedFlowhubModule, resolve_flowhub_module_children};

use super::contract::{
    validate_registered_contract, validate_root_contract, validate_unregistered_child_directories,
};
use super::filesystem::{count_glob_matches, last_module_segment};
use super::mermaid::validate_mermaid_case_files;
use super::model::{FlowhubCheckReport, FlowhubDiagnostic};
use super::source_pair::validate_org_bpmn_source_pairs;

pub(super) fn check_flowhub_root(root: &Path) -> Result<FlowhubCheckReport, QianjiError> {
    let mut diagnostics = Vec::new();
    let Some(root_manifest) = load_root_manifest_or_diagnostic(root, &mut diagnostics) else {
        return Ok(FlowhubCheckReport {
            target: root.to_path_buf(),
            checked_modules: 0,
            diagnostics,
        });
    };

    validate_root_contract(root, &root_manifest.contract, &mut diagnostics)?;
    let known_module_names = discover_all_flowhub_module_refs(root)?;
    let candidates = registered_root_candidates(root, &root_manifest.contract.register);
    if candidates.is_empty() {
        diagnostics.push(no_registered_modules_diagnostic(root));
        return Ok(FlowhubCheckReport {
            target: root.to_path_buf(),
            checked_modules: 0,
            diagnostics,
        });
    }

    let mut visited = BTreeSet::new();
    let checked_modules = validate_registered_candidates(
        &candidates,
        &known_module_names,
        &mut diagnostics,
        &mut visited,
    )?;

    Ok(FlowhubCheckReport {
        target: root.to_path_buf(),
        checked_modules,
        diagnostics,
    })
}

fn load_root_manifest_or_diagnostic(
    root: &Path,
    diagnostics: &mut Vec<FlowhubDiagnostic>,
) -> Option<crate::contracts::FlowhubRootManifest> {
    match load_flowhub_root_manifest(root.join("qianji.toml")) {
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
    }
}

fn registered_root_candidates(root: &Path, module_refs: &[String]) -> Vec<FlowhubModuleCandidate> {
    module_refs
        .iter()
        .map(|module_ref| module_candidate_from_ref(root, module_ref))
        .collect()
}

fn no_registered_modules_diagnostic(root: &Path) -> FlowhubDiagnostic {
    FlowhubDiagnostic {
        title: "No Flowhub modules".to_string(),
        location: root.to_path_buf(),
        problem: "the Flowhub root contract does not register any top-level graph modules"
            .to_string(),
        why_it_blocks: "Qianji cannot expose or validate any reusable Flowhub graph nodes"
            .to_string(),
        fix: "add at least one `contract.register` entry in the Flowhub root manifest".to_string(),
    }
}

fn validate_registered_candidates(
    candidates: &[FlowhubModuleCandidate],
    known_module_names: &[String],
    diagnostics: &mut Vec<FlowhubDiagnostic>,
    visited: &mut BTreeSet<String>,
) -> Result<usize, QianjiError> {
    candidates
        .iter()
        .filter(|candidate| candidate.manifest_path.is_file())
        .try_fold(0_usize, |checked_modules, candidate| {
            Ok(checked_modules
                + validate_candidate(candidate, known_module_names, diagnostics, visited)?)
        })
}

pub(super) fn check_flowhub_module(module_dir: &Path) -> Result<FlowhubCheckReport, QianjiError> {
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
        validate_org_bpmn_source_pairs(module, contract, diagnostics)?;
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
