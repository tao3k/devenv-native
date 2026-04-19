use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use xiuxian_config_core::load_toml_value_with_imports;

use crate::ClientContext;

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlConfig {
    #[serde(default)]
    link_graph: WendaoTomlLinkGraphConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlLinkGraphConfig {
    #[serde(default)]
    exclude_dirs: Vec<String>,
}

pub(super) fn configured_ignore_dirs(context: &ClientContext) -> Result<Vec<String>> {
    let Some(config_path) = resolve_config_path(context)? else {
        return Ok(Vec::new());
    };

    let merged = load_toml_value_with_imports(config_path.as_path()).with_context(|| {
        format!(
            "failed to load local get config from `{}`",
            config_path.display()
        )
    })?;
    let parsed: WendaoTomlConfig = merged.try_into().with_context(|| {
        format!(
            "failed to parse local get config from `{}`",
            config_path.display()
        )
    })?;

    let mut ignore_dirs = parsed
        .link_graph
        .exclude_dirs
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    ignore_dirs.sort();
    ignore_dirs.dedup();
    Ok(ignore_dirs)
}

fn resolve_config_path(context: &ClientContext) -> Result<Option<PathBuf>> {
    if let Some(config_path) = context.config_file() {
        let config_path = config_path.to_path_buf();
        if !config_path.is_file() {
            anyhow::bail!(
                "configured get config `{}` does not exist or is not a file",
                config_path.display()
            );
        }
        return Ok(Some(config_path));
    }

    let default_path = context.root().join("wendao.toml");
    if default_path.is_file() {
        Ok(Some(default_path))
    } else {
        Ok(None)
    }
}
