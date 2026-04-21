use super::input::{collect_changed_paths, resolve_root_dir};
use super::refresh::{build_index, execute_refresh, refresh_mode_label};
use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

/// Runtime `Wendao` refresh trigger.
///
/// This mechanism prefers incremental refresh from context-provided
/// changed paths and only falls back to full rebuild when required.
pub struct WendaoRefreshMechanism {
    /// Output context key for refresh telemetry.
    pub output_key: String,
    /// Context key containing changed paths (`string` or `string[]`).
    pub changed_paths_key: String,
    /// Optional context key resolving root directory.
    pub root_dir_key: Option<String>,
    /// Optional static root directory override.
    pub root_dir: Option<String>,
    /// Force full rebuild (ignores incremental preference).
    pub force_full: bool,
    /// Prefer incremental mode even when changed path count crosses threshold.
    pub prefer_incremental: bool,
    /// Allow full fallback when incremental refresh fails.
    pub allow_full_fallback: bool,
    /// Optional explicit threshold when not preferring incremental.
    pub full_rebuild_threshold: Option<usize>,
    /// Optional include directories for `LinkGraph` build.
    pub include_dirs: Vec<String>,
    /// Optional excluded directories for `LinkGraph` build.
    pub excluded_dirs: Vec<String>,
}

#[async_trait]
impl QianjiMechanism for WendaoRefreshMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        let changed_paths = collect_changed_paths(context, self.changed_paths_key.as_str());
        let root_dir = resolve_root_dir(
            context,
            self.root_dir.as_deref(),
            self.root_dir_key.as_deref(),
        )?;

        if changed_paths.is_empty() && !self.force_full {
            return Ok(QianjiOutput {
                data: json!({
                    self.output_key.clone(): {
                        "mode": "noop",
                        "changed_count": 0,
                        "force_full": false,
                        "fallback": false,
                        "root_dir": root_dir.display().to_string(),
                    }
                }),
                instruction: FlowInstruction::Continue,
            });
        }

        let mut index = build_index(root_dir.as_path(), &self.include_dirs, &self.excluded_dirs)?;
        let refresh = execute_refresh(
            &mut index,
            &changed_paths,
            self.force_full,
            self.prefer_incremental,
            self.allow_full_fallback,
            self.full_rebuild_threshold,
        )?;

        let changed_path_rows = changed_paths
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "mode": refresh_mode_label(refresh.mode),
                    "changed_count": changed_paths.len(),
                    "force_full": self.force_full,
                    "fallback": refresh.fallback,
                    "prefer_incremental": self.prefer_incremental,
                    "effective_threshold": refresh.threshold,
                    "root_dir": root_dir.display().to_string(),
                    "changed_paths": changed_path_rows,
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
