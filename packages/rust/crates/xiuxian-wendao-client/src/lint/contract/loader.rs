use super::assets::{
    MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID, markdown_lint_diagnostic_contract_snapshot,
};
use super::rule::RuleContract;
use super::snapshot::MarkdownLintDiagnosticContractSnapshot;
use super::validation::{known_rule_keys, normalize_rule_key, validate_snapshot};
use anyhow::{Context, Result, anyhow, bail};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

static DIAGNOSTIC_CONTRACT: OnceLock<MarkdownLintDiagnosticContract> = OnceLock::new();

pub(in crate::lint) fn diagnostic_contract() -> &'static MarkdownLintDiagnosticContract {
    DIAGNOSTIC_CONTRACT.get_or_init(|| match MarkdownLintDiagnosticContract::load() {
        Ok(contract) => contract,
        Err(error) => {
            panic!("embedded markdown lint diagnostic contract should parse: {error}")
        }
    })
}

pub(in crate::lint) struct MarkdownLintDiagnosticContract {
    rules: HashMap<String, RuleContract>,
}

impl MarkdownLintDiagnosticContract {
    fn load() -> Result<Self> {
        let raw_contract =
            markdown_lint_diagnostic_contract_snapshot(MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID)
                .ok_or_else(|| {
                    anyhow!("missing embedded markdown lint diagnostic contract snapshot")
                })?;
        let raw: MarkdownLintDiagnosticContractSnapshot = toml::from_str(raw_contract)
            .context("failed to parse markdown lint diagnostic contract snapshot TOML")?;
        validate_snapshot(&raw)?;
        let mut rules = HashMap::new();
        let mut seen_codes = HashSet::new();

        for raw_rule in raw.rules {
            let key = raw_rule.code.clone();
            let normalized_key = normalize_rule_key(key.as_str())
                .ok_or_else(|| anyhow!("unknown markdown lint rule `{key}` in contract"))?
                .to_string();
            if !seen_codes.insert(normalized_key.clone()) {
                bail!("duplicate markdown lint rule `{key}` in contract");
            }
            let contract = RuleContract::try_from(raw_rule)?;
            contract.validate(key.as_str())?;
            rules.insert(normalized_key, contract);
        }

        for rule_key in known_rule_keys() {
            if !rules.contains_key(rule_key) {
                bail!("markdown lint diagnostic contract is missing rule `{rule_key}`");
            }
        }

        Ok(Self { rules })
    }

    pub(in crate::lint) fn render_issue(
        &self,
        facts: &crate::lint::diagnostic::DiagnosticFacts,
    ) -> crate::lint::MarkdownLintIssue {
        let Some(rule) = self.rules.get(facts.rule_key()) else {
            panic!("all markdown lint rules should have diagnostic contracts");
        };
        rule.render_issue(facts)
    }
}
