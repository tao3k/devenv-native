use super::rule::RuleContract;
use super::validation::{validate_optional_field, validate_required_field};
use crate::lint::diagnostic::DiagnosticFacts;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ProblemStrategy {
    RedundantLabelProblem,
    DynamicProblemText,
}

impl ProblemStrategy {
    pub(super) fn render(self, facts: &DiagnosticFacts) -> String {
        match self {
            Self::RedundantLabelProblem => facts.redundant_problem(),
            Self::DynamicProblemText => facts.dynamic_problem_text().to_string(),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DetailStrategy {
    ParserMessage,
    Utf8ErrorMessage,
    DynamicDetailText,
}

impl DetailStrategy {
    pub(super) fn render(self, facts: &DiagnosticFacts) -> String {
        match self {
            Self::ParserMessage => facts.parser_message().to_string(),
            Self::Utf8ErrorMessage => format!(
                "Re-encode the file as UTF-8 before linting it: {}",
                facts.utf8_error()
            ),
            Self::DynamicDetailText => facts.dynamic_detail_text().to_string(),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OptionalTextStrategy {
    SourceLine,
    LinkLiteral,
    RewriteWithMarkdown,
    RewriteWikilinkOnly,
    DisplayLabelTip,
    DynamicFoundText,
    DynamicExpectedText,
    DynamicTipText,
}

impl OptionalTextStrategy {
    pub(super) fn render(self, facts: &DiagnosticFacts) -> Option<String> {
        match self {
            Self::SourceLine => facts.source().map(ToOwned::to_owned),
            Self::LinkLiteral => facts.link_literal().map(ToOwned::to_owned),
            Self::RewriteWithMarkdown => facts.rewrite_with_markdown(),
            Self::RewriteWikilinkOnly => facts.rewrite_wikilink_only(),
            Self::DisplayLabelTip => facts.display_label_tip(),
            Self::DynamicFoundText => facts.dynamic_found_text(),
            Self::DynamicExpectedText => facts.dynamic_expected_text(),
            Self::DynamicTipText => facts.dynamic_tip_text(),
        }
    }
}

pub(super) fn parse_optional_strategy<T>(raw: Option<&str>, field: &str) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct StrategyWrapper<T> {
        value: T,
    }

    raw.map(|value| {
        toml::from_str::<StrategyWrapper<T>>(&format!("value = \"{value}\""))
            .map(|wrapped| wrapped.value)
            .with_context(|| format!("unknown markdown lint {field} `{value}`"))
    })
    .transpose()
}

pub(super) fn validate_rule_contract(key: &str, contract: &RuleContract) -> Result<()> {
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
