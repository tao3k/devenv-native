use super::scope::{
    BpmnEngineError, BpmnEventKind, BpmnNodeIndex, BpmnProcessSpec, GatewayConditionError, Result,
    Value, evaluate_gateway_condition,
};

pub(crate) fn conditional_event_is_satisfied(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    variables: &Value,
) -> Result<bool> {
    let node = &process.nodes[node_index as usize];
    let event = process.event_for_node(node_index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    if event.kind != BpmnEventKind::Conditional {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "conditional_event_is_satisfied_non_conditional_event",
        });
    }
    let condition_expression = event
        .condition_expression
        .as_deref()
        .map(str::trim)
        .filter(|condition| !condition.is_empty())
        .ok_or_else(|| BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            element: "conditional_expression",
        })?;

    match evaluate_gateway_condition(condition_expression, variables) {
        Ok(ready) => Ok(ready),
        Err(GatewayConditionError::UnresolvedVariablePath(_)) => Ok(false),
        Err(GatewayConditionError::UnsupportedExpression) => {
            Err(BpmnEngineError::UnsupportedEventConfiguration {
                process_id: process.key.process_id.to_string(),
                node_id: node.bpmn_id.to_string(),
                detail: "unsupported_conditional_event_expression",
            })
        }
    }
}
