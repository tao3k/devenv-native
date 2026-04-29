use super::model::{
    TempContextEntry, TempContextExpression, TempInvocation, TempInvocationBinding,
    TempInvocationParameter, TempListExpression, TempLiteralExpression, TempRelationColumn,
    TempRelationExpression, TempRelationRow, TempTable,
};
use crate::dmn_model_api::{
    DmnContextEntry, DmnContextExpression, DmnHitPolicy, DmnInvocation, DmnInvocationBinding,
    DmnInvocationParameter, DmnListExpression, DmnLiteralExpression, DmnRelationColumn,
    DmnRelationExpression, DmnRelationRow, DmnSourceFile,
};
use crate::error::{BpmnEngineError, Result};

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

impl TempInvocation {
    pub(crate) fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnInvocation> {
        let invoked_expression = self
            .invoked_expression
            .map(|expression| expression.into_definition(source, decision_id))
            .transpose()?;
        let mut bindings = Vec::with_capacity(self.bindings.len());
        for binding in self.bindings {
            bindings.push(binding.into_definition(source, decision_id)?);
        }
        Ok(DmnInvocation::new(
            self.invocation_id,
            invoked_expression,
            bindings,
        ))
    }
}

impl TempInvocationBinding {
    fn into_definition(
        self,
        source: &DmnSourceFile,
        decision_id: &str,
    ) -> Result<DmnInvocationBinding> {
        let argument = self
            .argument
            .map(|expression| expression.into_definition(source, decision_id))
            .transpose()?;
        Ok(DmnInvocationBinding::new(
            self.binding_id,
            self.parameter.map(TempInvocationParameter::into_definition),
            argument,
        ))
    }
}

impl TempInvocationParameter {
    fn into_definition(self) -> DmnInvocationParameter {
        DmnInvocationParameter::new(self.parameter_id, self.name, self.type_ref)
    }
}

impl TempTable {
    pub(crate) fn empty_boxed_expression_table(decision_id: &str, expression_kind: &str) -> Self {
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
