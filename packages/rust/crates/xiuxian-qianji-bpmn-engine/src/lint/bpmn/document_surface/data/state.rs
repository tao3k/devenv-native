use crate::bpmn_model_api::BpmnDataStateSnapshot;
use serde_json::{Value, json};

pub(super) fn data_state_evidence(state: Option<&BpmnDataStateSnapshot>) -> Value {
    state.map_or(Value::Null, |state| {
        json!({
            "data_state_id": state.data_state_id,
            "name": state.name,
        })
    })
}
