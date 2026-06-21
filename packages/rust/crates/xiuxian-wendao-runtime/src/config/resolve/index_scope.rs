//! Index-scope configuration resolver for Wendao runtime search behavior.

use crate::config::LinkGraphIndexRuntimeConfig;
use crate::settings::{
    dedup_dirs, get_setting_bool, get_setting_string_list, normalize_relative_dir,
};
use serde_yaml::Value;
use std::path::Path;

/// Resolve `LinkGraph` index scope from merged `wendao` settings.
#[must_use]
pub fn resolve_link_graph_index_runtime_with_settings(
    root_dir: &Path,
    settings: &Value,
) -> LinkGraphIndexRuntimeConfig {
    let explicit_include = dedup_dirs(
        get_setting_string_list(settings, "sources.include_dirs")
            .into_iter()
            .filter_map(|item| normalize_relative_dir(&item))
            .collect(),
    );

    let include_dirs = if explicit_include.is_empty()
        && get_setting_bool(settings, "sources.include_dirs_auto").unwrap_or(true)
    {
        dedup_dirs(
            get_setting_string_list(settings, "sources.include_dirs_auto_candidates")
                .into_iter()
                .filter_map(|item| normalize_relative_dir(&item))
                .filter(|candidate| root_dir.join(candidate).is_dir())
                .collect(),
        )
    } else {
        explicit_include
    };

    let exclude_dirs = dedup_dirs(
        get_setting_string_list(settings, "sources.exclude_dirs")
            .into_iter()
            .filter_map(|item| normalize_relative_dir(&item))
            .filter(|value| !value.starts_with('.'))
            .collect(),
    );

    LinkGraphIndexRuntimeConfig {
        include_dirs,
        exclude_dirs,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/config/resolve/index_scope.rs"]
mod tests;
