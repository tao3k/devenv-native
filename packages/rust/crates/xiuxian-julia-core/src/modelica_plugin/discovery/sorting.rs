//! Stable Modelica repository record sorting helpers.

use std::collections::BTreeMap;
use std::path::Path;

use crate::modelica_plugin::pathing::path_components;

pub(crate) fn module_sort_key(
    path: &str,
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<(usize, String)> {
    if path == "package.mo" {
        return vec![(0, String::new())];
    }

    let components = path_components(path);
    let mut key = vec![(0, String::new())];
    let mut parent = String::new();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        let order_index = package_orders
            .get(parent.as_str())
            .and_then(|entries| entries.iter().position(|entry| entry == component))
            .unwrap_or(usize::MAX);
        key.push((order_index, (*component).to_string()));
        if parent.is_empty() {
            parent.push_str(component);
        } else {
            parent.push('/');
            parent.push_str(component);
        }
    }
    key
}

pub(crate) fn example_sort_key(
    path: &str,
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<(usize, String)> {
    let components = path_components(path);
    let mut key = vec![(0, String::new())];
    let mut parent = String::new();

    for component in components.iter().take(components.len().saturating_sub(1)) {
        let order_index = package_orders
            .get(parent.as_str())
            .and_then(|entries| entries.iter().position(|entry| entry == component))
            .unwrap_or(usize::MAX);
        key.push((order_index, (*component).to_string()));
        if parent.is_empty() {
            parent.push_str(component);
        } else {
            parent.push('/');
            parent.push_str(component);
        }
    }

    let example_name = Path::new(path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or(path);
    let order_index = package_orders
        .get(parent.as_str())
        .and_then(|entries| entries.iter().position(|entry| entry == example_name))
        .unwrap_or(usize::MAX);
    key.push((order_index, example_name.to_string()));
    key
}

pub(crate) fn doc_sort_key(
    path: &str,
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<(usize, String)> {
    let (source_path, variant_rank) = match path.split_once('#') {
        Some((source_path, "annotation.documentation")) => (source_path, 2usize),
        Some((source_path, suffix)) if suffix.starts_with("section.") => (source_path, 1usize),
        Some((source_path, _)) => (source_path, 1usize),
        None => (path, 0usize),
    };
    let components = path_components(source_path);
    let mut key = vec![(0, String::new())];
    let mut parent = String::new();

    for component in components.iter().take(components.len().saturating_sub(1)) {
        let order_index = package_orders
            .get(parent.as_str())
            .and_then(|entries| entries.iter().position(|entry| entry == component))
            .unwrap_or(usize::MAX);
        key.push((order_index, (*component).to_string()));
        if parent.is_empty() {
            parent.push_str(component);
        } else {
            parent.push('/');
            parent.push_str(component);
        }
    }

    let is_package = source_path.ends_with("package.mo");
    let leaf_name = doc_leaf_name(source_path);
    let leaf_order = if is_package {
        0
    } else {
        package_orders
            .get(parent.as_str())
            .and_then(|entries| entries.iter().position(|entry| entry == leaf_name.as_str()))
            .map_or(usize::MAX, |index| index.saturating_add(1))
    };
    key.push((leaf_order, leaf_name));
    key.push((variant_rank, String::new()));
    key
}

fn doc_leaf_name(path: &str) -> String {
    if path.ends_with("package.mo") {
        return Path::new(path)
            .parent()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("package")
            .to_string();
    }
    Path::new(path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or(path)
        .to_string()
}
