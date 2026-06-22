use super::model::{
    TempDecision, TempInformationRequirementReference, TempKnowledgeRequirementReference, TempTable,
};
use crate::dmn_model_api::{DmnDecisionDefinition, DmnDecisionRef, DmnSourceFile};
use crate::error::{BpmnEngineError, Result};

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
        invocation,
        information_requirements,
        knowledge_requirements,
    } = decision;
    let definition = match (
        table,
        literal_expression,
        list_expression,
        context_expression,
        relation_expression,
        invocation,
    ) {
        (Some(table), None, None, None, None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            table.into_definition(),
        ),
        (None, Some(literal_expression), None, None, None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "literalExpression")
                .into_definition(),
        )
        .with_literal_expression(literal_expression.into_definition(source, &decision_id)?),
        (None, None, Some(list_expression), None, None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "list").into_definition(),
        )
        .with_list_expression(list_expression.into_definition(source, &decision_id)?),
        (None, None, None, Some(context_expression), None, None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "context").into_definition(),
        )
        .with_context_expression(context_expression.into_definition(source, &decision_id)?),
        (None, None, None, None, Some(relation_expression), None) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "relation").into_definition(),
        )
        .with_relation_expression(relation_expression.into_definition(source, &decision_id)?),
        (None, None, None, None, None, Some(invocation)) => DmnDecisionDefinition::new(
            &source.source_id,
            DmnDecisionRef::new(&decision_id).with_source_id(&source.source_id),
            name,
            TempTable::empty_boxed_expression_table(&decision_id, "invocation").into_definition(),
        )
        .with_invocation(invocation.into_definition(source, &decision_id)?),
        (Some(_), Some(_), _, _, _, _)
        | (Some(_), _, Some(_), _, _, _)
        | (Some(_), _, _, Some(_), _, _)
        | (Some(_), _, _, _, Some(_), _)
        | (Some(_), _, _, _, _, Some(_))
        | (None, Some(_), Some(_), _, _, _)
        | (None, Some(_), _, Some(_), _, _)
        | (None, Some(_), _, _, Some(_), _)
        | (None, Some(_), _, _, _, Some(_))
        | (None, None, Some(_), Some(_), _, _)
        | (None, None, Some(_), _, Some(_), _)
        | (None, None, Some(_), _, _, Some(_))
        | (None, None, None, Some(_), Some(_), _)
        | (None, None, None, Some(_), _, Some(_))
        | (None, None, None, None, Some(_), Some(_)) => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "finalize_dmn_decision_mixed_executable_surfaces",
            });
        }
        (None, None, None, None, None, None) => {
            return Err(BpmnEngineError::MissingDmnDecisionTable {
                decision_id: decision_id.into(),
            });
        }
    };
    Ok(attach_requirement_contracts(
        definition,
        information_requirements,
        knowledge_requirements,
    ))
}

pub(crate) fn finalize_decision_definitions(
    source: &DmnSourceFile,
    decisions: Vec<TempDecision>,
) -> Result<Vec<DmnDecisionDefinition>> {
    if decisions.is_empty() {
        return Err(BpmnEngineError::MissingDmnDecision {
            source_id: (source.source_id.clone()).into(),
        });
    }

    decisions
        .into_iter()
        .map(|decision| finalize_decision_definition(source, decision))
        .collect()
}

fn attach_requirement_contracts(
    definition: DmnDecisionDefinition,
    information_requirements: Vec<TempInformationRequirementReference>,
    knowledge_requirements: Vec<TempKnowledgeRequirementReference>,
) -> DmnDecisionDefinition {
    let definition = if information_requirements.is_empty() {
        definition
    } else {
        definition.with_information_requirements(
            information_requirements
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    };
    if knowledge_requirements.is_empty() {
        definition
    } else {
        definition.with_knowledge_requirements(
            knowledge_requirements.into_iter().map(Into::into).collect(),
        )
    }
}
