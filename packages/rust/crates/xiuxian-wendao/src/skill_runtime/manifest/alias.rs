//! Native-alias compilation owned by Wendao skill runtime.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::{SkillManifest, SkillMetadata, SkillWorkflowType, ToolAnnotations};

/// Prefix used for skill runtime bindings.
pub const SKILL_BINDING_PREFIX: &str = "skill://";

/// Descriptor mapping a runtime binding id to a concrete native tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillBindingDescriptor {
    /// Unique binding identifier.
    pub binding_id: String,
    /// Target native tool name.
    pub target_tool_name: String,
    /// Expected workflow type.
    pub workflow_type: SkillWorkflowType,
}

/// Seed payload for native alias compilation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillNativeAliasSeed<Workflow> {
    /// Unique identifier for the manifest.
    pub manifest_id: String,
    /// Human-readable tool name.
    pub tool_name: String,
    /// Detailed tool description.
    pub description: String,
    /// Generic workflow type.
    pub workflow_type: Workflow,
    /// Target runtime binding identifier.
    pub binding_id: String,
    /// Opaque metadata dictionary.
    pub metadata: SkillMetadata,
    /// Tool annotation overrides.
    pub annotations: ToolAnnotations,
    /// Absolute path to the source manifest file.
    pub source_path: PathBuf,
}

/// Fully compiled alias spec for a runtime skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillNativeAliasSpec<Workflow> {
    /// Unique identifier for the manifest.
    pub manifest_id: String,
    /// Human-readable tool name.
    pub tool_name: String,
    /// Detailed tool description.
    pub description: String,
    /// Generic workflow type.
    pub workflow_type: Workflow,
    /// Target runtime binding identifier.
    pub binding_id: String,
    /// Opaque metadata dictionary.
    pub metadata: SkillMetadata,
    /// Resolved concrete native tool name.
    pub target_tool_name: String,
    /// Tool annotation overrides.
    pub annotations: ToolAnnotations,
    /// Absolute path to the source manifest file.
    pub source_path: PathBuf,
}

/// Compilation output for a batch of runtime aliases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillNativeAliasCompilation<Workflow> {
    /// Successfully compiled alias specifications.
    pub compiled_specs: Vec<SkillNativeAliasSpec<Workflow>>,
    /// Compilation errors or warnings.
    pub issues: Vec<String>,
}

/// Mount report for runtime aliases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillNativeAliasMountReport<Workflow> {
    /// Root directory for the mount operation.
    pub root: PathBuf,
    /// Paths to all discovered manifests.
    pub discovered_paths: Vec<PathBuf>,
    /// Specs that were successfully mounted.
    pub mounted_specs: Vec<SkillNativeAliasSpec<Workflow>>,
    /// Mount issues.
    pub issues: Vec<String>,
    /// Number of authorized manifests.
    pub authorized_count: usize,
    /// Number of ghost manifests.
    pub ghost_count: usize,
    /// Number of unauthorized manifests.
    pub unauthorized_count: usize,
}

impl<Workflow> SkillNativeAliasMountReport<Workflow> {
    /// Build a report rooted at the provided directory.
    #[must_use]
    pub fn from_root(root: &std::path::Path) -> Self {
        Self {
            root: root.to_path_buf(),
            discovered_paths: Vec::new(),
            mounted_specs: Vec::new(),
            issues: Vec::new(),
            authorized_count: 0,
            ghost_count: 0,
            unauthorized_count: 0,
        }
    }

    /// Total number of discovered manifest paths.
    #[must_use]
    pub fn discovered_count(&self) -> usize {
        self.discovered_paths.len()
    }

    /// Total number of authorized manifests.
    #[must_use]
    pub const fn authorized_count(&self) -> usize {
        self.authorized_count
    }

    /// Total number of ghost manifests.
    #[must_use]
    pub const fn ghost_count(&self) -> usize {
        self.ghost_count
    }

    /// Total number of unauthorized manifests.
    #[must_use]
    pub const fn unauthorized_count(&self) -> usize {
        self.unauthorized_count
    }

    /// Whether authority drift was detected.
    #[must_use]
    pub const fn has_authority_drift(&self) -> bool {
        self.ghost_count > 0 || self.unauthorized_count > 0
    }

    /// Whether the report indicates a critical failure.
    #[must_use]
    pub const fn is_critically_failed(&self) -> bool {
        self.ghost_count > 0
    }
}

/// Errors produced during runtime alias compilation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SkillNativeAliasCompileError {
    /// The provided binding id is not recognized by the registry.
    #[error("unknown skill binding id: {binding_id}")]
    UnknownBinding {
        /// The offending binding identifier.
        binding_id: String,
    },
}

/// Resolve a validated binding id to the concrete native tool name used at runtime.
///
/// # Errors
///
/// Returns an error when the binding id is unknown.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn resolve_skill_binding_target(
    binding_id: &str,
) -> Result<String, SkillNativeAliasCompileError> {
    let matched = skill_bindings()
        .into_iter()
        .find(|binding| binding.binding_id == binding_id)
        .map(|binding| binding.target_tool_name);
    matched.ok_or_else(|| SkillNativeAliasCompileError::UnknownBinding {
        binding_id: binding_id.to_string(),
    })
}

/// Return the current registry of runtime bindings.
#[must_use]
pub fn skill_bindings() -> Vec<SkillBindingDescriptor> {
    vec![
        SkillBindingDescriptor {
            binding_id: "xiuxian.native.zhixing.add".to_string(),
            target_tool_name: "task.add".to_string(),
            workflow_type: SkillWorkflowType::QianjiFlow,
        },
        SkillBindingDescriptor {
            binding_id: "xiuxian.native.zhixing.view".to_string(),
            target_tool_name: "agenda.view".to_string(),
            workflow_type: SkillWorkflowType::NativeDispatch,
        },
        SkillBindingDescriptor {
            binding_id: "xiuxian.native.spider".to_string(),
            target_tool_name: "web.crawl".to_string(),
            workflow_type: SkillWorkflowType::NativeDispatch,
        },
    ]
}

/// Compile a validated manifest payload into a runtime-ready native alias spec.
#[must_use]
pub fn compile_skill_native_alias<Workflow: Clone>(
    seed: SkillNativeAliasSeed<Workflow>,
) -> Option<SkillNativeAliasSpec<Workflow>> {
    try_compile_skill_native_alias(seed).ok()
}

/// Compile a validated manifest payload into a runtime-ready native alias spec.
///
/// # Errors
///
/// Returns an error when the manifest references an unknown runtime binding.
pub fn try_compile_skill_native_alias<Workflow: Clone>(
    seed: SkillNativeAliasSeed<Workflow>,
) -> Result<SkillNativeAliasSpec<Workflow>, SkillNativeAliasCompileError> {
    let target_tool_name = resolve_skill_binding_target(seed.binding_id.as_str())?;
    Ok(SkillNativeAliasSpec {
        manifest_id: seed.manifest_id,
        tool_name: seed.tool_name,
        description: seed.description,
        workflow_type: seed.workflow_type,
        binding_id: seed.binding_id,
        metadata: seed.metadata,
        target_tool_name,
        annotations: seed.annotations,
        source_path: seed.source_path,
    })
}

/// Compile a batch of validated skill manifests into native alias specs.
#[must_use]
pub fn compile_skill_manifest_aliases(
    manifests: Vec<SkillManifest>,
) -> SkillNativeAliasCompilation<SkillWorkflowType> {
    let mut compilation = SkillNativeAliasCompilation {
        compiled_specs: Vec::with_capacity(manifests.len()),
        issues: Vec::new(),
    };
    for manifest in manifests {
        let source_path = manifest.source_path.clone();
        let seed = SkillNativeAliasSeed {
            manifest_id: manifest.manifest_id,
            tool_name: manifest.tool_name,
            description: manifest.description,
            workflow_type: manifest.workflow_type,
            binding_id: manifest.binding_id,
            metadata: manifest.metadata,
            annotations: manifest.annotations,
            source_path: manifest.source_path,
        };
        match try_compile_skill_native_alias(seed) {
            Ok(spec) => compilation.compiled_specs.push(spec),
            Err(error) => compilation
                .issues
                .push(format!("{} -> {error}", source_path.display())),
        }
    }
    compilation
}
