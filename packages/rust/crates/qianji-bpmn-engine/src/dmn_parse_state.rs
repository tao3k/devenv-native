use super::unary;
use crate::dmn_model_api::{
    DmnDecisionDefinition, DmnDecisionRef, DmnDecisionTable, DmnHitPolicy, DmnInputClause,
    DmnInputEntry, DmnOutputClause, DmnOutputEntry, DmnRule, DmnSourceFile,
};
use crate::error::{BpmnEngineError, Result};

pub(crate) struct TempDecision {
    pub(crate) decision_id: String,
    pub(crate) name: Option<String>,
    pub(crate) table: Option<TempTable>,
}

pub(crate) struct TempTable {
    pub(crate) table_id: String,
    pub(crate) name: Option<String>,
    pub(crate) hit_policy: DmnHitPolicy,
    pub(crate) inputs: Vec<DmnInputClause>,
    pub(crate) outputs: Vec<DmnOutputClause>,
    pub(crate) rules: Vec<DmnRule>,
}

pub(crate) struct TempInput {
    pub(crate) input_id: String,
    pub(crate) label: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) expression: Option<String>,
}

pub(crate) struct TempOutput {
    pub(crate) output_id: String,
    pub(crate) label: Option<String>,
    pub(crate) name: Option<String>,
}

pub(crate) struct TempRule {
    pub(crate) rule_id: String,
    pub(crate) description: Option<String>,
    pub(crate) input_entries: Vec<DmnInputEntry>,
    pub(crate) output_entries: Vec<DmnOutputEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureTarget {
    InputExpression,
    RuleDescription,
    InputEntry,
    OutputEntry,
}

pub(crate) fn finalize_decision_definition(
    source: &DmnSourceFile,
    decision: Option<TempDecision>,
) -> Result<DmnDecisionDefinition> {
    let decision = decision.ok_or_else(|| BpmnEngineError::MissingDmnDecision {
        source_id: source.source_id.clone(),
    })?;
    let table = decision
        .table
        .ok_or_else(|| BpmnEngineError::MissingDmnDecisionTable {
            decision_id: decision.decision_id.clone(),
        })?;
    Ok(DmnDecisionDefinition::new(
        &source.source_id,
        DmnDecisionRef::new(&decision.decision_id).with_source_id(&source.source_id),
        decision.name,
        table.into_definition(),
    ))
}

pub(crate) fn finalize_input(
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
) {
    let Some(table) = current_table.as_mut() else {
        return;
    };
    let Some(input) = current_input.take() else {
        return;
    };
    table.inputs.push(DmnInputClause::new(
        input.input_id,
        input.label,
        input.name,
        input.expression,
    ));
}

pub(crate) fn finalize_output(
    current_table: &mut Option<TempTable>,
    current_output: &mut Option<TempOutput>,
) {
    let Some(table) = current_table.as_mut() else {
        return;
    };
    let Some(output) = current_output.take() else {
        return;
    };
    table.outputs.push(DmnOutputClause::new(
        output.output_id,
        output.label,
        output.name,
    ));
}

pub(crate) fn finalize_rule(
    source: &DmnSourceFile,
    current_table: &mut Option<TempTable>,
    current_rule: &mut Option<TempRule>,
) -> Result<()> {
    let Some(table) = current_table.as_mut() else {
        return Ok(());
    };
    let Some(rule) = current_rule.take() else {
        return Ok(());
    };
    if rule.input_entries.len() != table.inputs.len()
        || rule.output_entries.len() != table.outputs.len()
    {
        return Err(BpmnEngineError::InvalidDmnRuleArity {
            source_id: source.source_id.clone(),
            rule_id: rule.rule_id.clone(),
            expected_inputs: table.inputs.len(),
            actual_inputs: rule.input_entries.len(),
            expected_outputs: table.outputs.len(),
            actual_outputs: rule.output_entries.len(),
        });
    }
    table.rules.push(DmnRule::new(
        rule.rule_id,
        rule.description,
        rule.input_entries,
        rule.output_entries,
    ));
    Ok(())
}

pub(crate) fn finalize_input_entry(
    source: &DmnSourceFile,
    current_rule: &mut Option<TempRule>,
    capture_buffer: &str,
) -> Result<()> {
    let Some(rule) = current_rule.as_mut() else {
        return Ok(());
    };
    rule.input_entries.push(unary::parse_input_entry(
        source.source_id.as_str(),
        capture_buffer.trim(),
    )?);
    Ok(())
}

pub(crate) fn finalize_output_entry(
    source: &DmnSourceFile,
    current_rule: &mut Option<TempRule>,
    capture_buffer: &str,
) -> Result<()> {
    let Some(rule) = current_rule.as_mut() else {
        return Ok(());
    };
    rule.output_entries
        .push(DmnOutputEntry::new(unary::parse_literal(
            source.source_id.as_str(),
            capture_buffer.trim(),
        )?));
    Ok(())
}

pub(crate) fn hit_policy_from_attr(
    source: &DmnSourceFile,
    decision_id: &str,
    raw: Option<&str>,
) -> Result<DmnHitPolicy> {
    match raw.unwrap_or("UNIQUE").trim().to_ascii_uppercase().as_str() {
        "UNIQUE" => Ok(DmnHitPolicy::Unique),
        "COLLECT" => Ok(DmnHitPolicy::Collect),
        policy => Err(BpmnEngineError::UnsupportedDmnHitPolicy {
            source_id: source.source_id.clone(),
            decision_id: decision_id.to_string(),
            hit_policy: policy.to_string(),
        }),
    }
}

impl TempTable {
    pub(crate) fn into_definition(self) -> DmnDecisionTable {
        DmnDecisionTable::new(
            self.table_id,
            self.name,
            self.hit_policy,
            self.inputs,
            self.outputs,
            self.rules,
        )
    }
}
