use super::unary;
use crate::dmn_model_api::{
    DmnContextEntry, DmnContextExpression, DmnDecisionDefinition, DmnDecisionRef, DmnDecisionTable,
    DmnHitPolicy, DmnInformationRequirementReference, DmnInputClause, DmnInputEntry,
    DmnListExpression, DmnLiteralExpression, DmnOutputClause, DmnOutputEntry, DmnRelationColumn,
    DmnRelationExpression, DmnRelationRow, DmnRule, DmnSourceFile,
};
use crate::error::{BpmnEngineError, Result};

pub(crate) struct TempDecision {
    pub(crate) decision_id: String,
    pub(crate) name: Option<String>,
    pub(crate) table: Option<TempTable>,
    pub(crate) literal_expression: Option<TempLiteralExpression>,
    pub(crate) list_expression: Option<TempListExpression>,
    pub(crate) context_expression: Option<TempContextExpression>,
    pub(crate) relation_expression: Option<TempRelationExpression>,
    pub(crate) information_requirements: Vec<TempInformationRequirementReference>,
}

pub(crate) struct TempInformationRequirementReference {
    pub(crate) reference_kind: String,
    pub(crate) href: Option<String>,
}

pub(crate) struct TempLiteralExpression {
    pub(crate) expression_id: Option<String>,
    pub(crate) type_ref: Option<String>,
    pub(crate) text: Option<String>,
}

pub(crate) struct TempListExpression {
    pub(crate) list_id: Option<String>,
    pub(crate) items: Vec<TempLiteralExpression>,
}

pub(crate) struct TempContextExpression {
    pub(crate) context_id: Option<String>,
    pub(crate) entries: Vec<TempContextEntry>,
}

pub(crate) struct TempContextEntry {
    pub(crate) entry_id: Option<String>,
    pub(crate) variable_id: Option<String>,
    pub(crate) variable_name: Option<String>,
    pub(crate) literal_expression: Option<TempLiteralExpression>,
}

pub(crate) struct TempRelationExpression {
    pub(crate) relation_id: Option<String>,
    pub(crate) columns: Vec<TempRelationColumn>,
    pub(crate) rows: Vec<TempRelationRow>,
}

pub(crate) struct TempRelationColumn {
    pub(crate) column_id: String,
    pub(crate) name: Option<String>,
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempRelationRow {
    pub(crate) row_id: Option<String>,
    pub(crate) cells: Vec<TempLiteralExpression>,
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
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempOutput {
    pub(crate) output_id: String,
    pub(crate) label: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) type_ref: Option<String>,
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
    LiteralExpression,
    RuleDescription,
    InputEntry,
    OutputEntry,
}

pub(crate) fn finalize_decision_definition(
    source: &DmnSourceFile,
    decision: TempDecision,
) -> Result<DmnDecisionDefinition> {
    let TempDecision {
        decision_id,
        name,
        table,
        literal_expression,
        list_expression,
        context_expression,
        relation_expression,
        information_requirements,
    } = decision;
    let definition = match (
        table,
        literal_expression,
        list_expression,
        context_expression,
        relation_expression,
    ) {
        (Some(table), None, None, None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            table.into_definition(),
        ),
        (None, Some(literal_expression), None, None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "literalExpression")
                .into_definition(),
        )
        .with_literal_expression(literal_expression.into_definition(source, &decision_id)?),
        (None, None, Some(list_expression), None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "list").into_definition(),
        )
        .with_list_expression(list_expression.into_definition(source, &decision_id)?),
        (None, None, None, Some(context_expression), None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "context").into_definition(),
        )
        .with_context_expression(context_expression.into_definition(source, &decision_id)?),
        (None, None, None, None, Some(relation_expression)) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "relation").into_definition(),
        )
        .with_relation_expression(relation_expression.into_definition(source, &decision_id)?),
        (Some(_), Some(_), _, _, _)
        | (Some(_), _, Some(_), _, _)
        | (Some(_), _, _, Some(_), _)
        | (Some(_), _, _, _, Some(_))
        | (None, Some(_), Some(_), _, _)
        | (None, Some(_), _, Some(_), _)
        | (None, Some(_), _, _, Some(_))
        | (None, None, Some(_), Some(_), _)
        | (None, None, Some(_), _, Some(_))
        | (None, None, None, Some(_), Some(_)) => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "finalize_dmn_decision_mixed_executable_surfaces",
            });
        }
        (None, None, None, None, None) => {
            return Err(BpmnEngineError::MissingDmnDecisionTable { decision_id });
        }
    };
    if information_requirements.is_empty() {
        Ok(definition)
    } else {
        Ok(definition.with_information_requirements(
            information_requirements
                .into_iter()
                .map(Into::into)
                .collect(),
        ))
    }
}

impl From<TempInformationRequirementReference> for DmnInformationRequirementReference {
    fn from(value: TempInformationRequirementReference) -> Self {
        Self::new(value.reference_kind, value.href)
    }
}

impl TempLiteralExpression {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnLiteralExpression> {
        let text = self
            .text
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| BpmnEngineError::UnsupportedDmnLiteral {
                source_id: source.source_id.clone(),
                literal: format!("{decision_id}:<literalExpression>"),
            })?;
        Ok(DmnLiteralExpression::new(
            self.expression_id,
            self.type_ref,
            text,
        ))
    }
}

impl TempListExpression {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnListExpression> {
        let mut items = Vec::with_capacity(self.items.len());
        for item in self.items {
            items.push(item.into_definition(source, decision_id)?);
        }
        Ok(DmnListExpression::new(self.list_id, items))
    }
}

impl TempContextExpression {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnContextExpression> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            entries.push(entry.into_definition(source, decision_id)?);
        }
        Ok(DmnContextExpression::new(self.context_id, entries))
    }
}

impl TempContextEntry {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnContextEntry> {
        let literal_expression =
            self.literal_expression
                .ok_or(BpmnEngineError::UnsupportedOperation {
                    operation: "parse_dmn_context_entry_missing_literal_expression",
                })?;
        Ok(DmnContextEntry::new(
            self.entry_id,
            self.variable_id,
            self.variable_name,
            literal_expression.into_definition(source, decision_id)?,
        ))
    }
}

impl TempRelationExpression {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnRelationExpression> {
        let columns = self
            .columns
            .into_iter()
            .map(TempRelationColumn::into_definition)
            .collect();
        let mut rows = Vec::with_capacity(self.rows.len());
        for row in self.rows {
            rows.push(row.into_definition(source, decision_id)?);
        }
        Ok(DmnRelationExpression::new(self.relation_id, columns, rows))
    }
}

impl TempRelationColumn {
    fn into_definition(self) -> DmnRelationColumn {
        DmnRelationColumn::new(self.column_id, self.name, self.type_ref)
    }
}

impl TempRelationRow {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnRelationRow> {
        let mut cells = Vec::with_capacity(self.cells.len());
        for cell in self.cells {
            cells.push(cell.into_definition(source, decision_id)?);
        }
        Ok(DmnRelationRow::new(self.row_id, cells))
    }
}

impl TempTable {
    fn empty_boxed_expression_table(decision_id: &str, expression_kind: &str) -> Self {
        Self {
            table_id: format!("{decision_id}#{expression_kind}"),
            name: None,
            hit_policy: DmnHitPolicy::Unique,
            inputs: Vec::new(),
            outputs: Vec::new(),
            rules: Vec::new(),
        }
    }
}

pub(crate) fn finalize_decision_definitions(
    source: &DmnSourceFile,
    decisions: Vec<TempDecision>,
) -> Result<Vec<DmnDecisionDefinition>> {
    if decisions.is_empty() {
        return Err(BpmnEngineError::MissingDmnDecision {
            source_id: source.source_id.clone(),
        });
    }

    decisions
        .into_iter()
        .map(|decision| finalize_decision_definition(source, decision))
        .collect()
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
        input.type_ref,
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
        output.type_ref,
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
