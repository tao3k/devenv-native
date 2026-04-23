use crate::dmn_model_api::DmnDocumentSnapshot;

pub(super) fn decision_has_literal_expression(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.literal_expression_count > 0)
}

pub(super) fn decision_has_information_requirement(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.information_requirement_count > 0)
}

pub(super) fn decision_required_input_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(0, |decision| decision.required_input_count)
}

pub(super) fn decision_required_decision_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(0, |decision| decision.required_decision_count)
}

pub(super) fn decision_has_knowledge_requirement(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.knowledge_requirement_count > 0)
}

pub(super) fn decision_required_knowledge_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(0, |decision| decision.required_knowledge_count)
}

pub(super) fn decision_has_authority_requirement(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.authority_requirement_count > 0)
}

pub(super) fn decision_has_allowed_answers(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.allowed_answers_count > 0)
}

pub(super) fn decision_has_only_decision_maker(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| {
            decision.decision_maker_count > 0 && decision.decision_owner_count == 0
        })
}

pub(super) fn decision_has_mixed_decision_governance(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| {
            decision.decision_maker_count > 0 && decision.decision_owner_count > 0
        })
}

pub(super) fn decision_has_only_decision_owner(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| {
            decision.decision_maker_count == 0 && decision.decision_owner_count > 0
        })
}

pub(super) fn decision_maker_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(1, |decision| decision.decision_maker_count.max(1))
}

pub(super) fn decision_owner_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(1, |decision| decision.decision_owner_count.max(1))
}

pub(super) fn decision_required_authority_count(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .map_or(0, |decision| decision.required_authority_count)
}

pub(super) fn decision_has_context(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.context_count > 0)
}

pub(super) fn decision_has_invocation(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.invocation_count > 0)
}

pub(super) fn decision_has_relation(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.relation_count > 0)
}

pub(super) fn decision_has_function_definition(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.function_definition_count > 0)
}

pub(super) fn decision_has_list(decision_id: &str, snapshot: Option<&DmnDocumentSnapshot>) -> bool {
    snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .is_some_and(|decision| decision.list_count > 0)
}
