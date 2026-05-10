use crate::contract_feedback::{
    AdvisoryAuditRequest, ContractFinding, EvidenceKind, FindingEvidence, FindingSeverity,
    RoleAuditFinding,
};
use anyhow::{Result, anyhow};
use xiuxian_qianhuan::{
    InjectionSessionId, InjectionSnapshotId, InjectionSnapshotInput, InjectionTurnId,
    PersonaProfile, PromptContextBlock, PromptContextBlockId, PromptContextBlockInput,
    PromptContextCategory, PromptContextSource, PromptSessionScope, RoleMixProfile, RoleMixRole,
};

use super::helpers::{
    advisory_labels, advisory_summary, findings_summary, pack_summary, primary_finding,
    primary_finding_summary, primary_trace_id, role_mix_profile_id, runtime_trace_artifact_summary,
    runtime_trace_evidence, sanitize_identifier, snapshot_id,
};
use super::{QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan};

impl QianjiAdvisoryAuditExecutor {
    /// Build a typed multi-role advisory execution plan.
    ///
    /// # Errors
    ///
    /// Returns an error when any requested role cannot be resolved from the persona registry, when
    /// the role snapshot cannot be assembled, or when the generated `InjectionSnapshot` violates
    /// the configured injection policy.
    pub(crate) async fn build_plan_internal(
        &self,
        request: &AdvisoryAuditRequest,
    ) -> Result<QianjiAdvisoryExecutionPlan> {
        let resolved_roles = self.requested_roles(request);
        let role_mix = Self::build_role_mix(request, &resolved_roles);
        let session_id = Self::session_id(request);
        let primary_finding = primary_finding(&request.findings);
        let mut roles = Vec::with_capacity(resolved_roles.len());

        for (role_index, role_id) in resolved_roles.iter().enumerate() {
            let persona = self.resolve_persona(role_id)?;
            let blocks =
                Self::build_blocks(request, &session_id, &persona, primary_finding.as_ref());
            let narrative_blocks = blocks
                .iter()
                .map(|block| block.payload.clone())
                .collect::<Vec<_>>();
            let rendered_prompt = self
                .orchestrator
                .assemble_snapshot(&persona, narrative_blocks, "")
                .await
                .map_err(|error| {
                    anyhow!("failed to assemble advisory snapshot for '{role_id}': {error}")
                })?;
            let turn_id = u64::try_from(role_index + 1).map_err(|error| {
                anyhow!("role index overflow while preparing advisory plan: {error}")
            })?;
            let snapshot =
                xiuxian_qianhuan::InjectionSnapshot::from_blocks(InjectionSnapshotInput {
                    snapshot_id: InjectionSnapshotId::new(snapshot_id(request, role_id)),
                    session_id: InjectionSessionId::new(session_id.clone()),
                    turn_id: InjectionTurnId::new(turn_id),
                    policy: self.injection_policy.clone(),
                    role_mix: Some(role_mix.clone()),
                    blocks,
                });
            snapshot.validate().map_err(|error| {
                anyhow!("invalid advisory injection snapshot for role '{role_id}': {error}")
            })?;

            roles.push(QianjiAdvisoryRolePlan {
                role_id: role_id.clone(),
                persona_name: persona.name.clone(),
                snapshot,
                rendered_prompt,
            });
        }

        Ok(QianjiAdvisoryExecutionPlan {
            suite_id: request.suite_id.clone(),
            pack_id: request.pack_id.clone(),
            role_mix,
            roles,
        })
    }

    /// Build normalized scaffold findings for a previously prepared advisory plan.
    #[must_use]
    pub(crate) fn findings_from_plan(
        request: &AdvisoryAuditRequest,
        plan: &QianjiAdvisoryExecutionPlan,
    ) -> Vec<RoleAuditFinding> {
        let primary_finding = primary_finding(&request.findings);
        let trace_id = primary_trace_id(request);
        let runtime_trace_evidence = runtime_trace_evidence(request);

        plan.roles
            .iter()
            .map(|role_plan| {
                let mut finding = RoleAuditFinding::new(
                    role_plan.role_id.clone(),
                    primary_finding
                        .as_ref()
                        .map_or(FindingSeverity::Warning, |finding| finding.severity),
                    advisory_summary(role_plan, request.findings.len(), primary_finding.as_ref()),
                );

                if let Some(finding_rule_id) = primary_finding
                    .as_ref()
                    .map(|finding| finding.rule_id.clone())
                {
                    finding.rule_id = Some(finding_rule_id);
                }
                if let Some(ref top_finding) = primary_finding {
                    finding.confidence = top_finding.confidence;
                    finding.why_it_matters = if top_finding.why_it_matters.trim().is_empty() {
                        top_finding.summary.clone()
                    } else {
                        top_finding.why_it_matters.clone()
                    };
                    finding.remediation = if top_finding.remediation.trim().is_empty() {
                        "Run the live formal audit critique lane and attach the streamed evidence."
                            .to_string()
                    } else {
                        top_finding.remediation.clone()
                    };
                    finding.examples = top_finding.examples.clone();
                    finding.evidence.extend(top_finding.evidence.clone());
                } else {
                    finding.why_it_matters =
                        "Prepared advisory review without upstream deterministic findings."
                            .to_string();
                    finding.remediation =
                        "Provide deterministic findings before invoking multi-role advisory review."
                            .to_string();
                }

                finding.trace_id.clone_from(&trace_id);
                finding.evidence.extend(runtime_trace_evidence.clone());
                finding.evidence.push(FindingEvidence {
                    kind: EvidenceKind::DerivedInvariant,
                    path: None,
                    locator: Some(role_plan.snapshot.snapshot_id.as_ref().to_string()),
                    message: format!(
                        "Prepared Qianhuan advisory snapshot for '{}' with {} blocks and {} chars.",
                        role_plan.persona_name,
                        role_plan.snapshot.blocks.len(),
                        role_plan.snapshot.total_chars
                    ),
                });
                finding.labels = advisory_labels(request, &plan.role_mix, role_plan);

                finding
            })
            .collect()
    }

    fn requested_roles(&self, request: &AdvisoryAuditRequest) -> Vec<String> {
        if request.requested_roles.is_empty() {
            return vec![self.default_role_id.clone()];
        }

        let mut roles = Vec::with_capacity(request.requested_roles.len());
        for role_id in &request.requested_roles {
            if !roles.contains(role_id) {
                roles.push(role_id.clone());
            }
        }
        roles
    }

    fn build_role_mix(request: &AdvisoryAuditRequest, roles: &[String]) -> RoleMixProfile {
        RoleMixProfile {
            profile_id: role_mix_profile_id(request),
            roles: roles
                .iter()
                .map(|role_id| RoleMixRole {
                    role: role_id.clone(),
                    weight: 1.0,
                })
                .collect(),
            rationale: format!(
                "Prepared advisory role mix for contract suite '{}' and pack '{}'.",
                request.suite_id, request.pack_id
            ),
        }
    }

    fn session_id(request: &AdvisoryAuditRequest) -> String {
        request
            .collection_context
            .labels
            .get("session_id")
            .cloned()
            .unwrap_or_else(|| format!("contract-audit:{}:{}", request.suite_id, request.pack_id))
    }

    fn resolve_persona(&self, role_id: &str) -> Result<PersonaProfile> {
        self.registry.get(role_id).ok_or_else(|| {
            anyhow!("advisory role '{role_id}' is not registered in PersonaRegistry")
        })
    }

    fn build_blocks(
        request: &AdvisoryAuditRequest,
        session_id: &str,
        persona: &PersonaProfile,
        primary_finding: Option<&ContractFinding>,
    ) -> Vec<PromptContextBlock> {
        let mut blocks = vec![advisory_block(
            format!("{}:policy", sanitize_identifier(persona.id.as_str())),
            PromptContextSource::Policy,
            PromptContextCategory::Policy,
            1_000,
            session_id.to_string(),
            pack_summary(request),
            true,
        )];

        if !persona.style_anchors.is_empty() {
            blocks.push(advisory_block(
                format!("{}:anchors", sanitize_identifier(persona.id.as_str())),
                PromptContextSource::Policy,
                PromptContextCategory::Policy,
                950,
                session_id.to_string(),
                format!(
                    "Role anchors for {}: {}",
                    persona.name,
                    persona.style_anchors.join(", ")
                ),
                true,
            ));
        }

        blocks.push(advisory_block(
            format!("{}:findings", sanitize_identifier(persona.id.as_str())),
            PromptContextSource::RuntimeHint,
            PromptContextCategory::RuntimeHint,
            900,
            session_id.to_string(),
            findings_summary(&request.findings),
            false,
        ));

        if let Some(finding) = primary_finding {
            blocks.push(advisory_block(
                format!("{}:primary", sanitize_identifier(persona.id.as_str())),
                PromptContextSource::Knowledge,
                PromptContextCategory::Knowledge,
                875,
                session_id.to_string(),
                primary_finding_summary(finding),
                false,
            ));
        }

        let runtime_trace_summary = runtime_trace_artifact_summary(request);
        if !runtime_trace_summary.is_empty() {
            blocks.push(advisory_block(
                format!("{}:runtime", sanitize_identifier(persona.id.as_str())),
                PromptContextSource::RuntimeHint,
                PromptContextCategory::RuntimeHint,
                850,
                session_id.to_string(),
                runtime_trace_summary,
                false,
            ));
        }

        blocks
    }
}

fn advisory_block(
    block_id: impl Into<String>,
    source: PromptContextSource,
    category: PromptContextCategory,
    priority: u16,
    session_id: impl Into<String>,
    payload: impl Into<String>,
    anchor: bool,
) -> PromptContextBlock {
    PromptContextBlock::new(PromptContextBlockInput {
        block_id: PromptContextBlockId::new(block_id),
        source,
        category,
        priority,
        session_scope: PromptSessionScope::new(session_id),
        payload: payload.into(),
        anchor,
    })
}
