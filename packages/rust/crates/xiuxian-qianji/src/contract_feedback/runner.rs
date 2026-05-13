//! Qianji-owned contract-feedback suite runner.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::advisory::{AdvisoryAuditExecutor, AdvisoryAuditPolicy, AdvisoryAuditRequest};
use super::model::{CollectionContext, ContractExecutionMode, ContractFinding, ContractReport};
use super::rule_pack::ContractSuite;

/// Run configuration for one contract-feedback suite execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRunConfig {
    /// Top-level execution mode for the suite run.
    pub execution_mode: ContractExecutionMode,
    /// RFC 3339 or equivalent timestamp supplied by the caller.
    pub generated_at: String,
    /// Default advisory policy applied when no pack-specific override exists.
    pub default_advisory_policy: AdvisoryAuditPolicy,
    /// Pack-specific advisory policy overrides.
    pub advisory_policies: BTreeMap<String, AdvisoryAuditPolicy>,
}

impl Default for ContractRunConfig {
    fn default() -> Self {
        Self {
            execution_mode: ContractExecutionMode::Advisory,
            generated_at: String::new(),
            default_advisory_policy: AdvisoryAuditPolicy::default(),
            advisory_policies: BTreeMap::new(),
        }
    }
}

impl ContractRunConfig {
    /// Return the advisory policy that applies to the given pack identifier.
    #[must_use]
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub fn advisory_policy_for_pack(&self, pack_id: &str) -> AdvisoryAuditPolicy {
        self.advisory_policies
            .get(pack_id)
            .cloned()
            .unwrap_or_else(|| self.default_advisory_policy.clone())
    }

    /// Set one advisory policy override for a specific pack identifier.
    pub fn set_advisory_policy_for_pack(
        &mut self,
        pack_id: impl Into<String>,
        policy: AdvisoryAuditPolicy,
    ) {
        self.advisory_policies.insert(pack_id.into(), policy);
    }
}

/// Executes a contract-feedback suite and merges deterministic plus advisory findings.
pub struct ContractSuiteRunner<'a> {
    advisory_executor: &'a dyn AdvisoryAuditExecutor,
}

impl<'a> ContractSuiteRunner<'a> {
    /// Create a runner using the provided advisory executor.
    #[must_use]
    pub const fn new(advisory_executor: &'a dyn AdvisoryAuditExecutor) -> Self {
        Self { advisory_executor }
    }

    /// Execute one suite and return the merged contract report.
    ///
    /// # Errors
    ///
    /// Returns an error when deterministic collection or evaluation fails, or when the advisory
    /// executor returns an error for a triggered pack.
    pub async fn run(
        &self,
        suite: &ContractSuite,
        ctx: &CollectionContext,
        config: &ContractRunConfig,
    ) -> Result<ContractReport> {
        let mut findings = Vec::new();

        for rule_pack in suite.iter_rule_packs() {
            let descriptor = rule_pack.descriptor();
            let artifacts = rule_pack.collect(ctx)?;
            let deterministic_findings = rule_pack.evaluate(&artifacts)?;
            let advisory_policy = config.advisory_policy_for_pack(descriptor.id);

            findings.extend(deterministic_findings.clone());

            if should_trigger_advisory(&deterministic_findings, &advisory_policy) {
                let advisory_request = AdvisoryAuditRequest {
                    suite_id: suite.id().to_string(),
                    pack_id: descriptor.id.to_string(),
                    pack_version: descriptor.version.to_string(),
                    pack_domains: descriptor
                        .domains
                        .iter()
                        .map(|domain| (*domain).to_string())
                        .collect(),
                    findings: deterministic_findings,
                    artifacts,
                    collection_context: ctx.clone(),
                    requested_roles: advisory_policy.requested_roles,
                };
                let advisory_findings = self.advisory_executor.run(advisory_request).await?;
                findings.extend(advisory_findings.into_iter().enumerate().map(
                    |(ordinal, finding)| finding.into_contract_finding(descriptor.id, ordinal),
                ));
            }
        }

        Ok(ContractReport::from_findings(
            suite.id(),
            config.execution_mode,
            config.generated_at.clone(),
            findings,
        ))
    }
}

fn should_trigger_advisory(
    deterministic_findings: &[ContractFinding],
    policy: &AdvisoryAuditPolicy,
) -> bool {
    policy.enabled
        && deterministic_findings
            .iter()
            .any(|finding| finding.severity >= policy.min_severity)
}
