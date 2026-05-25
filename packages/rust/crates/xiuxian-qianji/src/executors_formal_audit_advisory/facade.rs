//! Facade surface for `xiuxian-qianji`.

use std::sync::Arc;

use crate::contract_feedback::{AdvisoryAuditExecutor, AdvisoryAuditRequest, RoleAuditFinding};
use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "advisory-prompt-pack-cache")]
use xiuxian_db_store::artifact_cache::ArtifactBlobCache;
use xiuxian_qianhuan::{
    InjectionPolicy, InjectionSnapshot, PersonaRegistry, RoleMixProfile, ThousandFacesOrchestrator,
};

const DEFAULT_ROLE_ID: &str = "strict_teacher";

/// Prompt-context pack read-through metrics for one advisory role.
#[cfg(feature = "advisory-prompt-pack-cache")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QianjiAdvisoryPromptPackArtifactReport {
    /// Whether the prompt-context pack was served from the shared artifact substrate.
    pub cache_hit: bool,
    /// Number of prompt-context pack bytes returned to the advisory planner.
    pub byte_len: usize,
}

#[cfg(feature = "advisory-prompt-pack-cache")]
impl QianjiAdvisoryPromptPackArtifactReport {
    /// Create prompt-context pack artifact-cache metrics.
    #[must_use]
    pub const fn new(cache_hit: bool, byte_len: usize) -> Self {
        Self {
            cache_hit,
            byte_len,
        }
    }
}

/// Planned advisory execution state for one resolved role.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiAdvisoryRolePlan {
    /// Stable role identifier requested by the contract runner.
    pub role_id: String,
    /// Friendly persona name resolved from the `Qianhuan` registry.
    pub persona_name: String,
    /// Typed `Qianhuan` injection snapshot prepared for this role.
    pub snapshot: InjectionSnapshot,
    /// Fully rendered system prompt snapshot prepared for later live execution.
    pub rendered_prompt: String,
    /// Optional prompt-context pack artifact-cache read-through metrics.
    #[cfg(feature = "advisory-prompt-pack-cache")]
    pub prompt_context_pack_artifact: Option<QianjiAdvisoryPromptPackArtifactReport>,
}

/// Planned multi-role advisory execution payload.
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiAdvisoryExecutionPlan {
    /// Stable suite identifier from the contract runner.
    pub suite_id: String,
    /// Rule-pack identifier under review.
    pub pack_id: String,
    /// Resolved role mix for this advisory pass.
    pub role_mix: RoleMixProfile,
    /// Per-role snapshot plan.
    pub roles: Vec<QianjiAdvisoryRolePlan>,
}

/// Qianji-side advisory executor scaffold backed by `Qianhuan` persona resolution.
///
/// This executor does not perform live LLM critique yet. Instead, it converts a
/// Qianji `AdvisoryAuditRequest` into:
/// - a `RoleMixProfile`
/// - typed `InjectionSnapshot` values for each role
/// - normalized `RoleAuditFinding` values that preserve deterministic evidence and trace context
///
/// The resulting bridge is immediately useful for testing and knowledge export while keeping the
/// future live `formal_audit + Zhenfa` critique lane compatible with the same request shape.
pub struct QianjiAdvisoryAuditExecutor {
    /// Orchestrator used to render per-role advisory prompt snapshots.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Persona registry used to resolve requested advisory roles.
    pub registry: Arc<PersonaRegistry>,
    /// Injection policy used to assemble typed advisory snapshots.
    pub injection_policy: InjectionPolicy,
    /// Fallback role used when the request does not specify any roles.
    pub default_role_id: String,
}

impl QianjiAdvisoryAuditExecutor {
    /// Create a new advisory executor bridge with default snapshot policy.
    #[must_use]
    pub fn new(
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
    ) -> Self {
        Self {
            orchestrator,
            registry,
            injection_policy: InjectionPolicy::default(),
            default_role_id: DEFAULT_ROLE_ID.to_string(),
        }
    }

    /// Override the injection policy used for advisory snapshot planning.
    #[must_use]
    pub fn with_injection_policy(mut self, injection_policy: InjectionPolicy) -> Self {
        self.injection_policy = injection_policy;
        self
    }

    /// Override the fallback role used when no explicit roles are requested.
    #[must_use]
    pub fn with_default_role_id(mut self, default_role_id: impl Into<String>) -> Self {
        self.default_role_id = default_role_id.into();
        self
    }

    /// Build a typed multi-role advisory execution plan.
    ///
    /// This preview surface prepares the resolved role mix and per-role `Qianhuan`
    /// snapshots without executing the live critique lane.
    ///
    /// # Errors
    ///
    /// Returns an error when any requested role cannot be resolved from the persona registry, when
    /// the role snapshot cannot be assembled, or when the generated `InjectionSnapshot` violates
    /// the configured injection policy.
    pub async fn build_plan(
        &self,
        request: &AdvisoryAuditRequest,
    ) -> Result<QianjiAdvisoryExecutionPlan> {
        self.build_plan_internal(request).await
    }

    /// Build a typed advisory execution plan while reading prompt-context packs
    /// through the shared artifact substrate.
    ///
    /// # Errors
    ///
    /// Returns an error when advisory planning fails or when the supplied
    /// artifact cache cannot read, build, or write a prompt-context pack.
    #[cfg(feature = "advisory-prompt-pack-cache")]
    pub async fn build_plan_with_prompt_context_pack_cache(
        &self,
        request: &AdvisoryAuditRequest,
        cache: &(dyn ArtifactBlobCache + Send + Sync),
    ) -> Result<QianjiAdvisoryExecutionPlan> {
        self.build_plan_internal_with_prompt_context_pack_cache(request, Some(cache))
            .await
    }
}

#[async_trait]
impl AdvisoryAuditExecutor for QianjiAdvisoryAuditExecutor {
    async fn run(&self, request: AdvisoryAuditRequest) -> Result<Vec<RoleAuditFinding>> {
        let plan = self.build_plan(&request).await?;
        Ok(Self::findings_from_plan(&request, &plan))
    }
}
