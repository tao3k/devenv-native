//! Template registry and rendering helpers for Qianhuan manifestation.

use crate::xml::SYSTEM_PROMPT_INJECTION_TAG;
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context, Tera};
use walkdir::WalkDir;

const DEFAULT_USER_TEMPLATE_SUBDIR: &str = "xiuxian-artisan-workshop/qianhuan/templates";
/// Canonical template file name used by orchestrator snapshot rendering.
pub const SYSTEM_PROMPT_TEMPLATE_NAME: &str = "system_prompt_injection.xml.j2";

/// Runtime-loaded renderer for the system prompt snapshot template.
///
/// The renderer supports hot-reload without process restart. On each render call,
/// it checks whether template files changed on disk and refreshes the in-memory
/// template cache when needed.
pub struct SystemPromptTemplateRenderer {
    template_dirs: Vec<PathBuf>,
    state: RwLock<TemplateRuntimeState>,
}

impl SystemPromptTemplateRenderer {
    /// Builds renderer from ordered directories.
    ///
    /// Later directories override earlier ones for duplicate template names.
    ///
    /// # Errors
    ///
    /// Returns an error when any provided path is invalid or the target template
    /// cannot be discovered.
    pub fn from_dirs(template_dirs: &[PathBuf]) -> Result<Self> {
        let state = build_runtime_state(template_dirs)?;
        Ok(Self {
            template_dirs: template_dirs.to_vec(),
            state: RwLock::new(state),
        })
    }

    /// Renders the system prompt snapshot template with the provided context.
    ///
    /// # Errors
    ///
    /// Returns an error when rendering fails.
    pub fn render(&self, mut context: Context) -> Result<String> {
        let _ = self.reload_templates_if_changed()?;
        context.insert("system_prompt_injection_tag", SYSTEM_PROMPT_INJECTION_TAG);
        let state = self
            .state
            .read()
            .map_err(|_| anyhow!("template renderer lock poisoned"))?;
        state
            .tera
            .render(SYSTEM_PROMPT_TEMPLATE_NAME, &context)
            .map_err(|error| anyhow!("template rendering failed: {error}"))
    }

    /// Returns watcher root directories for system prompt templates.
    #[must_use]
    pub fn template_watch_roots(&self) -> Vec<PathBuf> {
        self.template_dirs.clone()
    }

    /// Returns include patterns for system prompt templates.
    #[must_use]
    pub fn template_watch_patterns() -> Vec<String> {
        vec!["**/*.j2".to_string()]
    }

    /// Reloads templates when snapshot changes are detected.
    ///
    /// Returns `Ok(true)` when the renderer state was refreshed.
    ///
    /// # Errors
    ///
    /// Returns an error when snapshot capture or lock access fails.
    pub fn reload_templates_if_changed(&self) -> Result<bool> {
        let current_snapshot = capture_snapshot(&self.template_dirs)?;
        let should_reload = {
            let state = self
                .state
                .read()
                .map_err(|_| anyhow!("template renderer lock poisoned"))?;
            state.snapshot != current_snapshot
        };
        if !should_reload {
            return Ok(false);
        }

        match build_runtime_state(&self.template_dirs) {
            Ok(new_state) => {
                let mut state = self
                    .state
                    .write()
                    .map_err(|_| anyhow!("template renderer lock poisoned"))?;
                *state = new_state;
                log::info!(
                    "qianhuan templates hot-reloaded from [{}]",
                    self.template_dirs
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                Ok(true)
            }
            Err(error) => {
                log::warn!(
                    "qianhuan template hot-reload failed; keeping previous renderer state: {error}"
                );
                Ok(false)
            }
        }
    }
}

#[derive(Debug)]
struct TemplateRuntimeState {
    tera: Tera,
    snapshot: TemplateSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TemplateSnapshot {
    files: Vec<TemplateFileStamp>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TemplateFileStamp {
    path: PathBuf,
    modified_unix_millis: u128,
    size_bytes: u64,
    content_hash: u64,
}

fn build_runtime_state(template_dirs: &[PathBuf]) -> Result<TemplateRuntimeState> {
    let (discovered, scanned_dir_count) = discover_runtime_templates(template_dirs)?;

    if scanned_dir_count == 0 {
        return Err(anyhow!(
            "no template directories loaded for {SYSTEM_PROMPT_TEMPLATE_NAME}"
        ));
    }
    if discovered.is_empty() {
        return Err(anyhow!(
            "no jinja template files discovered in directories: {}",
            template_dirs
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    let mut tera = Tera::default();
    for (template_name, template_path) in &discovered {
        tera.add_template_file(template_path, Some(template_name.as_str()))
            .map_err(|error| {
                anyhow!(
                    "failed to add template '{}' from {}: {error}",
                    template_name,
                    template_path.display()
                )
            })?;
    }
    if !tera
        .get_template_names()
        .any(|name| name == SYSTEM_PROMPT_TEMPLATE_NAME)
    {
        return Err(anyhow!(
            "template '{}' not found in directories: {}",
            SYSTEM_PROMPT_TEMPLATE_NAME,
            template_dirs
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    Ok(TemplateRuntimeState {
        tera,
        snapshot: capture_snapshot(template_dirs)?,
    })
}

fn discover_runtime_templates(
    template_dirs: &[PathBuf],
) -> Result<(BTreeMap<String, PathBuf>, usize)> {
    template_dirs.iter().try_fold(
        (BTreeMap::new(), 0usize),
        |(mut discovered, scanned_dir_count), dir| {
            if !dir.exists() {
                return Ok((discovered, scanned_dir_count));
            }
            if !dir.is_dir() {
                return Err(anyhow!(
                    "template path is not a directory: {}",
                    dir.display()
                ));
            }
            collect_template_files(dir)?
                .into_iter()
                .try_for_each(|template_file| {
                    let template_name = template_name_from_path(dir, &template_file)?;
                    // Later directories override earlier ones.
                    discovered.insert(template_name, template_file);
                    Ok::<(), anyhow::Error>(())
                })?;
            Ok((discovered, scanned_dir_count + 1))
        },
    )
}

fn capture_snapshot(template_dirs: &[PathBuf]) -> Result<TemplateSnapshot> {
    let mut files = Vec::new();
    for dir in template_dirs {
        if !dir.exists() {
            continue;
        }
        if !dir.is_dir() {
            return Err(anyhow!(
                "template path is not a directory: {}",
                dir.display()
            ));
        }
        for template_file in collect_template_files(dir)? {
            let metadata = std::fs::metadata(&template_file).map_err(|error| {
                anyhow!(
                    "failed to stat template file {}: {error}",
                    template_file.display()
                )
            })?;
            let content_hash = hash_template_file(&template_file)?;
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let modified_unix_millis = modified
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            files.push(TemplateFileStamp {
                path: template_file,
                modified_unix_millis,
                size_bytes: metadata.len(),
                content_hash,
            });
        }
    }
    files.sort();
    Ok(TemplateSnapshot { files })
}

fn hash_template_file(path: &Path) -> Result<u64> {
    let contents = std::fs::read(path)
        .map_err(|error| anyhow!("failed to read template file {}: {error}", path.display()))?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Ok(hasher.finish())
}

/// Resolves ordered system prompt template directories.
///
/// Resolution order is:
/// 1) internal built-in resources,
/// 2) default user override directory,
/// 3) custom caller-provided directories.
#[must_use]
pub fn resolve_system_prompt_template_dirs(custom_template_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = vec![resolve_builtin_template_dir()];
    if let Some(default_user_dir) = resolve_default_user_template_dir() {
        dirs.push(default_user_dir);
    }
    dirs.extend(custom_template_dirs.iter().cloned());
    dedup_paths_preserve_order(dirs)
}

fn resolve_builtin_template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("qianhuan")
        .join("templates")
}

fn resolve_default_user_template_dir() -> Option<PathBuf> {
    if let Some(prj_config_home) = std::env::var("PRJ_CONFIG_HOME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Some(PathBuf::from(prj_config_home).join(DEFAULT_USER_TEMPLATE_SUBDIR));
    }
    std::env::var("HOME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| home.join(".config").join(DEFAULT_USER_TEMPLATE_SUBDIR))
}

fn dedup_paths_preserve_order(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().fold(Vec::new(), |mut unique, path| {
        if !unique.contains(&path) {
            unique.push(path);
        }
        unique
    })
}

fn collect_template_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|error| {
            anyhow!(
                "failed to walk template directory {}: {error}",
                path.display()
            )
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("j2"))
        {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

fn template_name_from_path(base_dir: &Path, file_path: &Path) -> Result<String> {
    let relative = file_path
        .strip_prefix(base_dir)
        .map_err(|error| anyhow!("failed to derive relative template path: {error}"))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
