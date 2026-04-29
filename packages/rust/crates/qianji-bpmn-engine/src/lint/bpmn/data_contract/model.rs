use super::{
    BpmnSourceFile, HashMap, HashSet, LintIssue, SequenceFlowContract,
    UndeclaredGatewayConditionIssue, declares_gateway_variable, gateway_condition_variable_path,
    undeclared_gateway_condition_output_issue,
};

#[derive(Default)]
pub(super) struct ProcessContract {
    pub(super) id: String,
    pub(super) task_outputs: HashMap<String, HashSet<String>>,
    pub(super) gateways: HashSet<String>,
    pub(super) flows: Vec<SequenceFlowContract>,
}

impl ProcessContract {
    pub(super) fn undeclared_gateway_condition_output_issues(
        self,
        source: &BpmnSourceFile,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        for flow in &self.flows {
            if !self.gateways.contains(&flow.source_ref) {
                continue;
            }
            let Some(condition) = flow.condition.as_deref() else {
                continue;
            };
            let Some(variable_path) = gateway_condition_variable_path(condition) else {
                continue;
            };
            let producer_ids = self.direct_upstream_task_ids(&flow.source_ref);
            if producer_ids.is_empty() {
                continue;
            }
            let producer_outputs = producer_ids
                .iter()
                .flat_map(|producer_id| {
                    self.task_outputs
                        .get(producer_id)
                        .into_iter()
                        .flat_map(|outputs| outputs.iter().cloned())
                })
                .collect::<HashSet<_>>();
            if producer_outputs.is_empty()
                || declares_gateway_variable(&producer_outputs, &variable_path)
            {
                continue;
            }
            issues.push(undeclared_gateway_condition_output_issue(
                UndeclaredGatewayConditionIssue {
                    source,
                    process_id: &self.id,
                    gateway_id: &flow.source_ref,
                    target_id: &flow.target_ref,
                    condition,
                    variable_path: &variable_path,
                    producer_ids: &producer_ids,
                    producer_outputs: &producer_outputs,
                    condition_span: flow.condition_span.clone(),
                },
            ));
        }
        issues
    }

    fn direct_upstream_task_ids(&self, gateway_id: &str) -> Vec<String> {
        self.flows
            .iter()
            .filter(|flow| flow.target_ref == gateway_id)
            .filter(|flow| self.task_outputs.contains_key(&flow.source_ref))
            .map(|flow| flow.source_ref.clone())
            .collect()
    }
}
