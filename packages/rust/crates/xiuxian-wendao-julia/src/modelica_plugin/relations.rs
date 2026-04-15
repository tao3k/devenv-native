use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use xiuxian_wendao_core::repo_intelligence::{
    DocRecord, ExampleRecord, ModuleRecord, RelationKind, RelationRecord, SymbolRecord,
};

use super::discovery::{
    containing_module_name, modules_by_qualified_name, path_components, qualified_module_name,
};
use super::types::CollectedDoc;

pub(crate) fn collect_relation_records(
    repo_id: &str,
    root_package_name: &str,
    modules: &[ModuleRecord],
    module_lookup: &BTreeMap<String, ModuleRecord>,
    symbols: &[SymbolRecord],
    examples: &[ExampleRecord],
    docs: &[CollectedDoc],
) -> Vec<RelationRecord> {
    let mut relation_keys = BTreeSet::new();
    let mut relations = Vec::new();

    for module in modules {
        if module.qualified_name == root_package_name {
            continue;
        }
        if let Some((parent, _)) = module.qualified_name.rsplit_once('.')
            && let Some(parent_module) = module_lookup.get(parent)
        {
            push_relation(
                &mut relations,
                &mut relation_keys,
                RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: parent_module.module_id.clone(),
                    target_id: module.module_id.clone(),
                    kind: RelationKind::Contains,
                },
            );
        }
    }

    for symbol in symbols {
        if let Some(module_id) = symbol.module_id.as_ref() {
            push_relation(
                &mut relations,
                &mut relation_keys,
                RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: module_id.clone(),
                    target_id: symbol.symbol_id.clone(),
                    kind: RelationKind::Declares,
                },
            );
        }
    }

    let root_module_id = module_lookup
        .get(root_package_name)
        .map(|module| module.module_id.clone());

    for example in examples {
        let target_module = target_module_for_example(example.path.as_str(), root_package_name)
            .and_then(|qualified_name| module_lookup.get(qualified_name.as_str()))
            .map(|module| module.module_id.clone())
            .or_else(|| root_module_id.clone());
        if let Some(target_id) = target_module {
            push_relation(
                &mut relations,
                &mut relation_keys,
                RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: example.example_id.clone(),
                    target_id,
                    kind: RelationKind::ExampleOf,
                },
            );
        }
    }

    for doc in docs {
        for target_id in &doc.target_ids {
            push_relation(
                &mut relations,
                &mut relation_keys,
                RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: doc.record.doc_id.clone(),
                    target_id: target_id.clone(),
                    kind: RelationKind::Documents,
                },
            );
        }
    }

    relations
}

pub(crate) fn build_incremental_doc_relations(
    repo_id: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    docs: &[DocRecord],
) -> Vec<RelationRecord> {
    let Some(root_module) = modules
        .iter()
        .filter(|module| module.path.ends_with("package.mo"))
        .min_by_key(|module| path_components(module.path.as_str()).len())
    else {
        return Vec::new();
    };

    let module_lookup = modules_by_qualified_name(modules);
    let root_package_name = root_module.qualified_name.as_str();
    let root_module_id = Some(root_module.module_id.as_str());

    let mut relation_keys = BTreeSet::new();
    let mut relations = Vec::new();
    for doc in docs {
        let (source_path, suffix) = match doc.path.split_once('#') {
            Some((source_path, suffix)) => (source_path, Some(suffix)),
            None => (doc.path.as_str(), None),
        };
        let is_julia_source = Path::new(source_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("jl"));
        if is_julia_source || doc.format.as_deref() == Some("julia_docstring") {
            continue;
        }

        let target_ids = if matches!(suffix, Some("annotation.documentation")) {
            doc_targets_for_annotation_doc(
                source_path,
                root_package_name,
                &module_lookup,
                symbols,
                root_module_id,
            )
        } else {
            doc_targets_for_file_doc(
                source_path,
                root_package_name,
                &module_lookup,
                root_module_id,
            )
        };
        for target_id in target_ids {
            push_relation(
                &mut relations,
                &mut relation_keys,
                RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: doc.doc_id.clone(),
                    target_id,
                    kind: RelationKind::Documents,
                },
            );
        }
    }

    relations
}

fn push_relation(
    relations: &mut Vec<RelationRecord>,
    relation_keys: &mut BTreeSet<String>,
    relation: RelationRecord,
) {
    let key = format!(
        "{}::{}::{}::{:?}",
        relation.repo_id, relation.source_id, relation.target_id, relation.kind
    );
    if relation_keys.insert(key) {
        relations.push(relation);
    }
}

fn target_module_for_example(example_path: &str, root_package_name: &str) -> Option<String> {
    let components = path_components(example_path);
    let examples_index = components
        .iter()
        .position(|component| *component == "Examples")?;
    if examples_index == 0 {
        return Some(root_package_name.to_string());
    }
    let mut qualified = root_package_name.to_string();
    for component in &components[..examples_index] {
        qualified.push('.');
        qualified.push_str(component);
    }
    Some(qualified)
}

pub(crate) fn doc_targets_for_file_doc(
    relative_path: &str,
    root_package_name: &str,
    module_lookup: &BTreeMap<String, ModuleRecord>,
    root_module_id: Option<&str>,
) -> Vec<String> {
    let is_readme = Path::new(relative_path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.to_ascii_lowercase().starts_with("readme"));
    if is_readme {
        let mut target_ids = BTreeSet::new();
        if let Some(root_module_id) = root_module_id {
            target_ids.insert(root_module_id.to_string());
        }
        return target_ids.into_iter().collect();
    }

    if is_users_guide_path(relative_path) {
        return users_guide_target_ids(
            relative_path,
            root_package_name,
            module_lookup,
            root_module_id,
        )
        .into_iter()
        .collect();
    }

    Vec::new()
}

pub(crate) fn doc_targets_for_annotation_doc(
    relative_path: &str,
    root_package_name: &str,
    module_lookup: &BTreeMap<String, ModuleRecord>,
    symbols: &[SymbolRecord],
    root_module_id: Option<&str>,
) -> Vec<String> {
    if is_users_guide_path(relative_path) {
        return users_guide_target_ids(
            relative_path,
            root_package_name,
            module_lookup,
            root_module_id,
        )
        .into_iter()
        .collect();
    }

    let mut target_ids = BTreeSet::new();
    if relative_path.ends_with("package.mo") {
        if let Some(module_qualified_name) = qualified_module_name(root_package_name, relative_path)
        {
            if let Some(module) = module_lookup.get(module_qualified_name.as_str()) {
                target_ids.insert(module.module_id.clone());
            }
        } else if let Some(root_module_id) = root_module_id {
            target_ids.insert(root_module_id.to_string());
        }
        return target_ids.into_iter().collect();
    }

    let file_stem = Path::new(relative_path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str);
    if let Some(file_stem) = file_stem
        && let Some(symbol) = symbols
            .iter()
            .find(|symbol| symbol.path == relative_path && symbol.name == file_stem)
    {
        target_ids.insert(symbol.symbol_id.clone());
    }
    if target_ids.is_empty()
        && let Some(module_qualified_name) =
            containing_module_name(root_package_name, relative_path)
        && let Some(module) = module_lookup.get(module_qualified_name.as_str())
    {
        target_ids.insert(module.module_id.clone());
    }
    target_ids.into_iter().collect()
}

pub(crate) fn annotation_doc_title(relative_path: &str, symbols: &[SymbolRecord]) -> String {
    let source_path = relative_path
        .strip_suffix("#annotation.documentation")
        .unwrap_or(relative_path);
    if source_path.ends_with("package.mo") {
        return format!(
            "{} documentation",
            Path::new(source_path)
                .parent()
                .and_then(Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
                .filter(|name| !name.is_empty())
                .unwrap_or("package")
        );
    }
    let file_stem = Path::new(source_path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("symbol");
    let title = symbols
        .iter()
        .find(|symbol| symbol.path == source_path && symbol.name == file_stem)
        .map_or(file_stem, |symbol| symbol.name.as_str());
    format!("{title} documentation")
}

fn push_module_target(
    target_ids: &mut BTreeSet<String>,
    module_lookup: &BTreeMap<String, ModuleRecord>,
    module_qualified_name: &str,
) {
    if let Some(module) = module_lookup.get(module_qualified_name) {
        target_ids.insert(module.module_id.clone());
    }
}

fn users_guide_owner_module_name(relative_path: &str, root_package_name: &str) -> Option<String> {
    let components = path_components(relative_path);
    let users_guide_index = components
        .iter()
        .position(|component| *component == "UsersGuide")?;
    if users_guide_index == 0 {
        return Some(root_package_name.to_string());
    }
    let mut qualified = root_package_name.to_string();
    for component in &components[..users_guide_index] {
        qualified.push('.');
        qualified.push_str(component);
    }
    Some(qualified)
}

fn users_guide_target_ids(
    relative_path: &str,
    root_package_name: &str,
    module_lookup: &BTreeMap<String, ModuleRecord>,
    root_module_id: Option<&str>,
) -> BTreeSet<String> {
    let mut target_ids = BTreeSet::new();
    if let Some(owner_module_name) = users_guide_owner_module_name(relative_path, root_package_name)
    {
        push_module_target(&mut target_ids, module_lookup, owner_module_name.as_str());
    }
    for users_guide_module_name in
        users_guide_hierarchy_module_names(relative_path, root_package_name)
    {
        push_module_target(
            &mut target_ids,
            module_lookup,
            users_guide_module_name.as_str(),
        );
    }
    if target_ids.is_empty()
        && let Some(root_module_id) = root_module_id
    {
        target_ids.insert(root_module_id.to_string());
    }
    target_ids
}

fn is_users_guide_path(relative_path: &str) -> bool {
    path_components(relative_path).contains(&"UsersGuide")
}

fn users_guide_hierarchy_module_names(relative_path: &str, root_package_name: &str) -> Vec<String> {
    let components = path_components(relative_path);
    let Some(users_guide_index) = components
        .iter()
        .position(|component| *component == "UsersGuide")
    else {
        return Vec::new();
    };
    let module_components = &components[..components.len().saturating_sub(1)];
    let mut names = Vec::new();
    for end in (users_guide_index + 1)..=module_components.len() {
        let mut qualified = root_package_name.to_string();
        for component in &module_components[..end] {
            qualified.push('.');
            qualified.push_str(component);
        }
        names.push(qualified);
    }
    names
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/relations.rs"]
mod tests;
