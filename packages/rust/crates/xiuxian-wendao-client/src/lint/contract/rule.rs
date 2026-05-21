use super::snapshot::MarkdownLintRuleContractSnapshot;
use super::strategy::{
    DetailStrategy, OptionalTextStrategy, ProblemStrategy, parse_optional_strategy,
};
use super::validation::{validate_optional_field, validate_required_field};
use crate::lint::MarkdownLintIssue;
use crate::lint::diagnostic::DiagnosticFacts;
use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(super) struct RuleContract {
    pub(super) problem: Option<String>,
    pub(super) problem_strategy: Option<ProblemStrategy>,
    pub(super) detail: Option<String>,
    pub(super) detail_strategy: Option<DetailStrategy>,
    pub(super) found: Option<String>,
    pub(super) found_strategy: Option<OptionalTextStrategy>,
    pub(super) expected: Option<String>,
    pub(super) expected_strategy: Option<OptionalTextStrategy>,
    pub(super) tip: Option<String>,
    pub(super) tip_strategy: Option<OptionalTextStrategy>,
}

fn validate_rule_contract(key: &str, contract: &RuleContract) -> Result<()> {
    validate_required_field(
        key,
        "problem",
        contract.problem.is_some(),
        contract.problem_strategy.is_some(),
    )?;
    validate_required_field(
        key,
        "detail",
        contract.detail.is_some(),
        contract.detail_strategy.is_some(),
    )?;
    validate_optional_field(
        key,
        "found",
        contract.found.is_some(),
        contract.found_strategy.is_some(),
    )?;
    validate_optional_field(
        key,
        "expected",
        contract.expected.is_some(),
        contract.expected_strategy.is_some(),
    )?;
    validate_optional_field(
        key,
        "tip",
        contract.tip.is_some(),
        contract.tip_strategy.is_some(),
    )?;
    Ok(())
}

impl TryFrom<MarkdownLintRuleContractSnapshot> for RuleContract {
    type Error = anyhow::Error;

    fn try_from(value: MarkdownLintRuleContractSnapshot) -> Result<Self> {
        Ok(Self {
            problem: value.problem,
            problem_strategy: parse_optional_strategy(
                value.problem_strategy.as_deref(),
                "problem_strategy",
            )?,
            detail: value.detail,
            detail_strategy: parse_optional_strategy(
                value.detail_strategy.as_deref(),
                "detail_strategy",
            )?,
            found: value.found,
            found_strategy: parse_optional_strategy(
                value.found_strategy.as_deref(),
                "found_strategy",
            )?,
            expected: value.expected,
            expected_strategy: parse_optional_strategy(
                value.expected_strategy.as_deref(),
                "expected_strategy",
            )?,
            tip: value.tip,
            tip_strategy: parse_optional_strategy(value.tip_strategy.as_deref(), "tip_strategy")?,
        })
    }
}

impl RuleContract {
    pub(super) fn validate(&self, key: &str) -> Result<()> {
        validate_rule_contract(key, self)
    }

    pub(super) fn render_issue(&self, facts: &DiagnosticFacts) -> MarkdownLintIssue {
        MarkdownLintIssue {
            code: facts.rule_key().to_string(),
            kind: facts.kind().into(),
            problem: self.render_problem(facts),
            message: self.render_detail(facts),
            line: facts.line(),
            column: facts.column(),
            target: facts.target().map(ToOwned::to_owned),
            target_title: facts.target_title().map(ToOwned::to_owned),
            target_heading: facts.target_heading().map(ToOwned::to_owned),
            found: self.render_found(facts),
            expected: self.render_expected(facts),
            source: facts.source().map(ToOwned::to_owned),
            tip: self.render_tip(facts),
        }
    }

    fn render_problem(&self, facts: &DiagnosticFacts) -> String {
        if let Some(problem) = &self.problem {
            return problem.clone();
        }

        let Some(strategy) = self.problem_strategy else {
            panic!("validated problem strategy should exist");
        };
        strategy.render(facts)
    }

    fn render_detail(&self, facts: &DiagnosticFacts) -> String {
        if let Some(detail) = &self.detail {
            return detail.clone();
        }

        let Some(strategy) = self.detail_strategy else {
            panic!("validated detail strategy should exist");
        };
        strategy.render(facts)
    }

    fn render_found(&self, facts: &DiagnosticFacts) -> Option<String> {
        self.found.clone().or_else(|| {
            self.found_strategy
                .and_then(|strategy| strategy.render(facts))
        })
    }

    fn render_expected(&self, facts: &DiagnosticFacts) -> Option<String> {
        self.expected.clone().or_else(|| {
            self.expected_strategy
                .and_then(|strategy| strategy.render(facts))
        })
    }

    fn render_tip(&self, facts: &DiagnosticFacts) -> Option<String> {
        self.tip.clone().or_else(|| {
            self.tip_strategy
                .and_then(|strategy| strategy.render(facts))
        })
    }
}
