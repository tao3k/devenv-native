pub(super) struct ActiveGatewayFlow {
    pub(super) gateway_id: String,
    pub(super) process_id: String,
    pub(super) depth: usize,
}

#[derive(Debug, Clone)]
pub(super) struct UnsupportedGatewayCondition {
    pub(super) process_id: String,
    pub(super) gateway_id: String,
    pub(super) condition: String,
}

pub(super) struct UnsupportedGatewayConditionGroup {
    pub(super) process_id: String,
    pub(super) gateway_id: String,
    pub(super) conditions: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AmbiguousBooleanPathKind {
    CountLike,
    ContentLike,
}

#[derive(Debug, Clone)]
pub(super) struct StaticInteractionChoiceOutput {
    pub(super) task_id: String,
    pub(super) output: String,
    pub(super) choice_values: Vec<String>,
}
