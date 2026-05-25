//! Safe Modelica package-overlay metadata helpers.

use std::path::Path;

use crate::modelica_plugin::parsing::{
    PackageOverlayMetadata, parse_safe_package_overlay_metadata,
    parse_safe_root_package_overlay_metadata,
};

pub(crate) fn package_overlay_expected_name(
    root_package_name: &str,
    relative_package_path: &str,
) -> Option<String> {
    if relative_package_path == "package.mo" {
        return Some(root_package_name.to_string());
    }
    Path::new(relative_package_path)
        .parent()?
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_string)
}

pub(crate) fn safe_package_overlay_metadata_for_relative_path(
    relative_package_path: &str,
    contents: &str,
    root_package_name: &str,
) -> Option<PackageOverlayMetadata> {
    if !relative_package_path.ends_with("package.mo") {
        return None;
    }
    let expected_package_name =
        package_overlay_expected_name(root_package_name, relative_package_path)?;
    parse_safe_package_overlay_metadata(contents, expected_package_name.as_str()).or_else(|| {
        if relative_package_path == "package.mo" {
            parse_safe_root_package_overlay_metadata(contents)
        } else {
            None
        }
    })
}
