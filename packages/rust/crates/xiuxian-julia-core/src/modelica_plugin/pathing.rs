//! Modelica repository path-to-owner identity helpers.

use std::collections::BTreeMap;
use std::path::Path;

use xiuxian_wendao_core::repo_intelligence::ModuleRecord;

pub(crate) fn modules_by_qualified_name(
    modules: &[ModuleRecord],
) -> BTreeMap<String, ModuleRecord> {
    modules
        .iter()
        .cloned()
        .map(|module| (module.qualified_name.clone(), module))
        .collect()
}

pub(crate) fn qualified_module_name(
    root_package_name: &str,
    relative_package_path: &str,
) -> Option<String> {
    if relative_package_path == "package.mo" {
        return Some(root_package_name.to_string());
    }
    let mut qualified = root_package_name.to_string();
    let relative_dir = Path::new(relative_package_path).parent()?;
    for component in relative_dir.components() {
        let std::path::Component::Normal(part) = component else {
            continue;
        };
        qualified.push('.');
        qualified.push_str(part.to_str()?);
    }
    Some(qualified)
}

pub(crate) fn containing_module_name(
    root_package_name: &str,
    relative_path: &str,
) -> Option<String> {
    let parent = Path::new(relative_path).parent()?;
    if parent.as_os_str().is_empty() {
        return Some(root_package_name.to_string());
    }
    let mut qualified = root_package_name.to_string();
    for component in parent.components() {
        let std::path::Component::Normal(part) = component else {
            continue;
        };
        qualified.push('.');
        qualified.push_str(part.to_str()?);
    }
    Some(qualified)
}

pub(crate) fn path_components(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|component| !component.is_empty())
        .collect()
}
