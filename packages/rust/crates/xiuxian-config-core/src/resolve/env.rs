//! Import path environment expansion for project-local config files.

use crate::{ConfigCoreError, normalize_config_home, resolve_cache_home, resolve_data_home};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub(super) struct ImportPathContext {
    project_root: Option<PathBuf>,
    config_home: Option<PathBuf>,
}

impl ImportPathContext {
    pub(super) fn from_process_environment() -> Self {
        Self::default()
    }

    pub(super) fn from_paths(project_root: Option<&Path>, config_home: Option<&Path>) -> Self {
        Self {
            project_root: project_root.map(Path::to_path_buf),
            config_home: normalize_config_home(project_root, config_home),
        }
    }

    pub(super) fn resolve_import_path(
        &self,
        source_path: Option<&Path>,
        raw_import_path: &str,
    ) -> Result<PathBuf, ConfigCoreError> {
        let expanded = self.expand_import_path(source_path, raw_import_path)?;
        let candidate = Path::new(expanded.as_str());
        if candidate.is_absolute() {
            return Ok(candidate.to_path_buf());
        }

        Ok(source_path
            .and_then(Path::parent)
            .map_or_else(|| candidate.to_path_buf(), |base| base.join(candidate)))
    }

    fn expand_import_path(
        &self,
        source_path: Option<&Path>,
        raw_import_path: &str,
    ) -> Result<String, ConfigCoreError> {
        let source = source_label(source_path);
        let chars = raw_import_path.chars().collect::<Vec<_>>();
        let mut expanded = String::with_capacity(raw_import_path.len());
        let mut cursor = 0usize;

        while cursor < chars.len() {
            if chars[cursor] != '$' {
                expanded.push(chars[cursor]);
                cursor += 1;
                continue;
            }

            if cursor + 1 >= chars.len() {
                expanded.push('$');
                cursor += 1;
                continue;
            }

            if chars[cursor + 1] == '{' {
                let Some(close_index) = chars[cursor + 2..]
                    .iter()
                    .position(|ch| *ch == '}')
                    .map(|offset| cursor + 2 + offset)
                else {
                    return Err(ConfigCoreError::InvalidImports {
                        path: source,
                        message: "import path contains an unterminated `${...}` variable"
                            .to_string(),
                    });
                };

                let variable = chars[cursor + 2..close_index].iter().collect::<String>();
                if variable.is_empty() {
                    return Err(ConfigCoreError::InvalidImports {
                        path: source,
                        message: "import path contains an empty `${...}` variable".to_string(),
                    });
                }

                expanded.push_str(self.lookup_env(source_path, variable.as_str())?.as_str());
                cursor = close_index + 1;
                continue;
            }

            if !is_env_var_start(chars[cursor + 1]) {
                expanded.push('$');
                cursor += 1;
                continue;
            }

            let mut end = cursor + 2;
            while end < chars.len() && is_env_var_continue(chars[end]) {
                end += 1;
            }

            let variable = chars[cursor + 1..end].iter().collect::<String>();
            expanded.push_str(self.lookup_env(source_path, variable.as_str())?.as_str());
            cursor = end;
        }

        Ok(expanded)
    }

    fn lookup_env(
        &self,
        source_path: Option<&Path>,
        variable: &str,
    ) -> Result<String, ConfigCoreError> {
        let resolved = match variable {
            "PRJ_ROOT" => self
                .project_root
                .clone()
                .or_else(|| env_path(variable))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_CONFIG_HOME" => self
                .config_home
                .clone()
                .or_else(|| env_path(variable))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_DATA_HOME" => env_path(variable)
                .or_else(|| resolve_data_home(self.project_root.as_deref()))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_CACHE_HOME" => env_path(variable)
                .or_else(|| resolve_cache_home(self.project_root.as_deref()))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_RUNTIME_DIR" => env_path(variable)
                .or_else(|| self.project_root.as_ref().map(|root| root.join(".run")))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_PATH" => env_path(variable)
                .or_else(|| self.project_root.as_ref().map(|root| root.join(".bin")))
                .map(|path| path_to_string(path.as_path())),
            "PRJ_SKILLS_DIR" => env_path(variable)
                .or_else(|| self.project_root.as_ref().map(|root| root.join("skills")))
                .map(|path| path_to_string(path.as_path())),
            _ => std::env::var(variable)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        };

        resolved.ok_or_else(|| ConfigCoreError::UnresolvedEnvironmentVariable {
            path: source_label(source_path),
            variable: variable.to_string(),
        })
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn source_label(source_path: Option<&Path>) -> String {
    source_path.map_or_else(
        || "<embedded>".to_string(),
        |path| path.display().to_string(),
    )
}

fn is_env_var_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_env_var_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
