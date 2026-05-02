//! Qianji-owned deterministic rule-pack interfaces for contract feedback.

use anyhow::Result;

use super::model::{CollectedArtifacts, CollectionContext, ContractFinding, FindingMode};

/// Stable descriptor for one rule pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePackDescriptor {
    /// Stable pack identifier.
    pub id: &'static str,
    /// Pack version string.
    pub version: &'static str,
    /// Logical domains covered by the pack.
    pub domains: &'static [&'static str],
    /// Default finding mode for the pack.
    pub default_mode: FindingMode,
}

/// Deterministic rule-pack interface.
pub trait RulePack: Send + Sync {
    /// Return the stable descriptor for the pack.
    fn descriptor(&self) -> RulePackDescriptor;

    /// Collect artifacts needed by the pack.
    ///
    /// # Errors
    ///
    /// Returns an error when artifact collection fails.
    fn collect(&self, ctx: &CollectionContext) -> Result<CollectedArtifacts>;

    /// Evaluate collected artifacts and emit findings.
    ///
    /// # Errors
    ///
    /// Returns an error when evaluation fails.
    fn evaluate(&self, artifacts: &CollectedArtifacts) -> Result<Vec<ContractFinding>>;
}

/// One contract-feedback suite containing multiple rule packs.
#[derive(Default)]
pub struct ContractSuite {
    id: String,
    version: String,
    rule_packs: Vec<Box<dyn RulePack>>,
}

impl ContractSuite {
    /// Create an empty contract-feedback suite.
    #[must_use]
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            rule_packs: Vec::new(),
        }
    }

    /// Return the suite identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the suite version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Register one rule pack into the suite.
    pub fn register_rule_pack(&mut self, rule_pack: Box<dyn RulePack>) {
        self.rule_packs.push(rule_pack);
    }

    /// Return the number of registered rule packs.
    #[must_use]
    pub fn rule_pack_count(&self) -> usize {
        self.rule_packs.len()
    }

    pub(crate) fn iter_rule_packs(&self) -> impl Iterator<Item = &dyn RulePack> {
        self.rule_packs.iter().map(Box::as_ref)
    }
}
