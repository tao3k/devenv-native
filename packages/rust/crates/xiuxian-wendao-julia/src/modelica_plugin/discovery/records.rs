//! Modelica repository module, symbol, import, and example record collection.

use std::collections::{BTreeMap, BTreeSet};

use xiuxian_wendao_core::repo_intelligence::{
    ExampleRecord, ImportRecord, ModuleRecord, RegisteredRepository, RepoIntelligenceError,
    SymbolRecord,
};

use super::overlay::safe_package_overlay_metadata_for_relative_path;
use super::snapshot::{RepositoryFileEntry, RepositorySnapshot};
use super::sorting::{example_sort_key, module_sort_key};
use super::surface::{RepositorySurface, is_api_surface_path};
use crate::modelica_plugin::parsing::{
    parse_imports_for_repository, parse_symbol_declarations_for_repository,
};
use crate::modelica_plugin::pathing::{containing_module_name, qualified_module_name};

pub(crate) fn collect_module_records(
    repo_id: &str,
    root_package_name: &str,
    package_files: &[&RepositoryFileEntry],
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<ModuleRecord> {
    let mut modules = package_files
        .iter()
        .filter_map(|entry| {
            let relative = entry.relative_path.as_str();
            if relative != "package.mo" && entry.surface == RepositorySurface::Support {
                return None;
            }
            let qualified_name = qualified_module_name(root_package_name, relative)?;
            Some(ModuleRecord {
                repo_id: repo_id.to_string(),
                module_id: module_id(repo_id, qualified_name.as_str()),
                qualified_name,
                path: relative.to_string(),
            })
        })
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| {
        module_sort_key(left.path.as_str(), package_orders)
            .cmp(&module_sort_key(right.path.as_str(), package_orders))
            .then_with(|| left.qualified_name.cmp(&right.qualified_name))
            .then_with(|| left.path.cmp(&right.path))
    });
    modules
}

pub(crate) fn collect_symbol_records(
    repository: &RegisteredRepository,
    repo_id: &str,
    snapshot: &RepositorySnapshot,
    root_package_name: &str,
    modules: &BTreeMap<String, ModuleRecord>,
) -> Result<Vec<SymbolRecord>, RepoIntelligenceError> {
    let mut symbols = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in snapshot.entries() {
        let Some(contents) = entry.modelica_contents.as_deref() else {
            continue;
        };
        if entry.surface != RepositorySurface::Api {
            continue;
        }
        let Some(module_qualified_name) =
            containing_module_name(root_package_name, entry.relative_path.as_str())
        else {
            continue;
        };
        let module_id = modules
            .get(module_qualified_name.as_str())
            .map(|module| module.module_id.clone());
        if safe_package_overlay_metadata_for_relative_path(
            entry.relative_path.as_str(),
            contents,
            root_package_name,
        )
        .is_some()
        {
            continue;
        }

        for declaration in parse_symbol_declarations_for_repository(
            repository,
            entry.relative_path.as_str(),
            contents,
        )? {
            let qualified_name = format!("{module_qualified_name}.{}", declaration.name);
            let symbol_id = format!("repo:{repo_id}:symbol:{qualified_name}");
            if !seen.insert(symbol_id.clone()) {
                continue;
            }
            symbols.push(SymbolRecord {
                repo_id: repo_id.to_string(),
                symbol_id,
                module_id: module_id.clone(),
                name: declaration.name,
                qualified_name,
                kind: declaration.kind,
                path: entry.relative_path.clone(),
                line_start: declaration.line_start,
                line_end: declaration.line_end,
                signature: Some(declaration.signature),
                audit_status: None,
                verification_state: None,
                attributes: declaration.attributes,
            });
        }
    }

    symbols.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
    Ok(symbols)
}

pub(crate) fn collect_import_records(
    repository: &RegisteredRepository,
    repo_id: &str,
    snapshot: &RepositorySnapshot,
    root_package_name: &str,
    modules: &BTreeMap<String, ModuleRecord>,
) -> Result<Vec<ImportRecord>, RepoIntelligenceError> {
    let mut imports = Vec::new();

    for entry in snapshot.entries() {
        let Some(contents) = entry.modelica_contents.as_deref() else {
            continue;
        };
        if entry.surface == RepositorySurface::Support {
            continue;
        }
        imports.extend(collect_import_records_for_file(
            repository,
            repo_id,
            entry.relative_path.as_str(),
            entry.relative_path.as_str(),
            contents,
            root_package_name,
            modules,
        )?);
    }

    imports.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.source_module.cmp(&right.source_module))
            .then_with(|| left.import_name.cmp(&right.import_name))
            .then_with(|| left.target_package.cmp(&right.target_package))
    });
    Ok(imports)
}

pub(crate) fn collect_import_records_for_file(
    repository: &RegisteredRepository,
    repo_id: &str,
    relative_within_root: &str,
    record_path: &str,
    contents: &str,
    root_package_name: &str,
    modules: &BTreeMap<String, ModuleRecord>,
) -> Result<Vec<ImportRecord>, RepoIntelligenceError> {
    let Some(module_qualified_name) =
        containing_module_name(root_package_name, relative_within_root)
    else {
        return Ok(Vec::new());
    };
    let source_module_id = modules.get(module_qualified_name.as_str()).map_or_else(
        || module_id(repo_id, module_qualified_name.as_str()),
        |module| module.module_id.clone(),
    );
    let mut imports = Vec::new();
    let mut seen = BTreeSet::new();

    let parsed_imports = if is_api_surface_path(relative_within_root) {
        if let Some(metadata) = safe_package_overlay_metadata_for_relative_path(
            relative_within_root,
            contents,
            root_package_name,
        ) {
            metadata.imports
        } else {
            parse_imports_for_repository(repository, relative_within_root, contents)?
        }
    } else {
        parse_imports_for_repository(repository, relative_within_root, contents)?
    };

    for parsed_import in parsed_imports {
        let source_module = parsed_import.name.clone();
        let import_name = parsed_import
            .alias
            .clone()
            .unwrap_or_else(|| import_leaf_name(source_module.as_str()));
        let target_package = source_module
            .split('.')
            .next()
            .unwrap_or(source_module.as_str())
            .to_string();
        let resolved_id = modules
            .get(source_module.as_str())
            .map(|module| module.module_id.clone());
        let kind_key = match parsed_import.kind {
            xiuxian_wendao_core::repo_intelligence::ImportKind::Symbol => "symbol",
            xiuxian_wendao_core::repo_intelligence::ImportKind::Module => "module",
            xiuxian_wendao_core::repo_intelligence::ImportKind::Reexport => "reexport",
        };
        let import_key = (
            record_path.to_string(),
            source_module.clone(),
            import_name.clone(),
            kind_key,
        );
        if !seen.insert(import_key) {
            continue;
        }
        imports.push(ImportRecord {
            repo_id: repo_id.to_string(),
            module_id: source_module_id.clone(),
            path: record_path.to_string(),
            import_name,
            target_package,
            source_module,
            kind: parsed_import.kind,
            line_start: parsed_import.line_start,
            resolved_id,
            attributes: parsed_import.attributes,
        });
    }

    imports.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.source_module.cmp(&right.source_module))
            .then_with(|| left.import_name.cmp(&right.import_name))
            .then_with(|| left.target_package.cmp(&right.target_package))
    });
    Ok(imports)
}

fn import_leaf_name(import_path: &str) -> String {
    import_path
        .rsplit('.')
        .next()
        .unwrap_or(import_path)
        .trim()
        .to_string()
}

pub(crate) fn collect_example_records(
    repo_id: &str,
    snapshot: &RepositorySnapshot,
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<ExampleRecord> {
    let mut examples = Vec::new();
    for entry in snapshot.entries() {
        if entry.modelica_contents.is_none() {
            continue;
        }
        if entry.surface != RepositorySurface::Example {
            continue;
        }
        if entry
            .absolute_path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            == Some("package.mo")
        {
            continue;
        }
        let title = entry
            .absolute_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("example")
            .to_string();
        examples.push(ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: format!("repo:{repo_id}:example:{}", entry.relative_path),
            title,
            path: entry.relative_path.clone(),
            summary: None,
        });
    }
    examples.sort_by(|left, right| {
        example_sort_key(left.path.as_str(), package_orders)
            .cmp(&example_sort_key(right.path.as_str(), package_orders))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.path.cmp(&right.path))
    });
    examples
}

fn module_id(repo_id: &str, qualified_name: &str) -> String {
    format!("repo:{repo_id}:module:{qualified_name}")
}
