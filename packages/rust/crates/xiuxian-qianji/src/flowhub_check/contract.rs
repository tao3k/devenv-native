use std::collections::BTreeSet;
use std::path::Path;

use globset::Glob;

use crate::contracts::FlowhubStructureContract;
use crate::error::QianjiError;
use crate::flowhub::discover::FlowhubDiscoveredModule;

use super::api::FlowhubDiagnostic;
use super::filesystem::{
    count_root_glob_matches, discover_immediate_child_directories, is_glob_pattern,
};

pub(super) fn validate_root_contract(
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

pub(super) fn validate_registered_contract(
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

pub(super) fn validate_unregistered_child_directories(
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

pub(super) fn expanded_required_entries(contract: &FlowhubStructureContract) -> Vec<String> {
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

pub(super) fn mermaid_file_is_contracted(
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
