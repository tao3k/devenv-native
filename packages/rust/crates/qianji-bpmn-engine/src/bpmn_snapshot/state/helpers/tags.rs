pub(in crate::bpmn_snapshot::state) fn is_collaboration_container(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some("collaboration" | "globalConversation" | "choreography" | "globalChoreographyTask")
    )
}

pub(in crate::bpmn_snapshot::state) fn is_conversation_node_tag(tag: &str) -> bool {
    matches!(tag, "conversation" | "subConversation" | "callConversation")
}

pub(in crate::bpmn_snapshot::state) fn is_choreography_activity_tag(tag: &str) -> bool {
    matches!(
        tag,
        "choreographyTask" | "subChoreography" | "callChoreography"
    )
}

pub(in crate::bpmn_snapshot::state) fn is_data_association_tag(tag: &str) -> bool {
    matches!(tag, "dataInputAssociation" | "dataOutputAssociation")
}

pub(in crate::bpmn_snapshot::state) fn is_artifact_container(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some(
            "collaboration"
                | "globalConversation"
                | "choreography"
                | "globalChoreographyTask"
                | "process"
                | "subProcess"
                | "transaction"
                | "subChoreography"
        )
    )
}

pub(in crate::bpmn_snapshot::state) fn is_global_task_tag(tag: &str) -> bool {
    matches!(
        tag,
        "globalTask"
            | "globalBusinessRuleTask"
            | "globalManualTask"
            | "globalScriptTask"
            | "globalUserTask"
    )
}

pub(in crate::bpmn_snapshot::state) fn is_resource_role_tag(tag: &str) -> bool {
    matches!(
        tag,
        "resourceRole" | "performer" | "humanPerformer" | "potentialOwner"
    )
}

pub(in crate::bpmn_snapshot::state) fn is_flow_element_metadata_owner_tag(tag: &str) -> bool {
    matches!(
        tag,
        "adHocSubProcess"
            | "boundaryEvent"
            | "businessRuleTask"
            | "callActivity"
            | "complexGateway"
            | "dataObject"
            | "dataObjectReference"
            | "dataStoreReference"
            | "endEvent"
            | "eventBasedGateway"
            | "exclusiveGateway"
            | "inclusiveGateway"
            | "intermediateCatchEvent"
            | "intermediateThrowEvent"
            | "manualTask"
            | "parallelGateway"
            | "receiveTask"
            | "scriptTask"
            | "sendTask"
            | "sequenceFlow"
            | "serviceTask"
            | "startEvent"
            | "subProcess"
            | "task"
            | "transaction"
            | "userTask"
    )
}
