//! Static construct-card registry split by construct family.
//!
//! Start with `agent` for host-owned service task cards; sibling modules own
//! interaction, gateway, and DMN cards.

use super::{ConstructCard, ConstructIndexEntry, ConstructLintMapping};

mod agent;
mod dmn;
mod gateway;
mod interaction;
mod loop_progress;
mod multi_instance;

const GATEWAY_LINT: &[ConstructLintMapping] = &[ConstructLintMapping {
    diagnostic: "bpmn.unsupported_gateway_configuration",
    repair: "Move rich logic into an upstream serviceTask or DMN decision that outputs a declared boolean, route on the plain variable, and use a gateway default branch only when the gateway has at least two outgoing flows.",
}];

const TASK_CONFIG_LINT: &[ConstructLintMapping] = &[
    ConstructLintMapping {
        diagnostic: "bpmn.missing_host_task_contract",
        repair: "Add a qianji-owned extension config with prompt, inputs, outputs, and implementation metadata expected by the selected host adapter.",
    },
    ConstructLintMapping {
        diagnostic: "bpmn.unsupported_qianji_interaction_type",
        repair: "Use one supported qianji interaction type: input, confirm, choice, or choice_input. Use input for plain free-form text and choice_input for option selection plus free-form feedback.",
    },
    ConstructLintMapping {
        diagnostic: "bpmn.ambiguous_qianji_interaction_outputs",
        repair: "Set userTask qianji:outputs to exactly the qianji:result output, then derive any secondary variables in a following serviceTask that consumes that answer.",
    },
];

const DMN_LINT: &[ConstructLintMapping] = &[ConstructLintMapping {
    diagnostic: "dmn.invalid_decision_table",
    repair: "Keep one explicit decision id, declared inputs, typed outputs, and rules that match the declared hit policy.",
}];

const PARALLEL_MULTI_INSTANCE_LINT: &[ConstructLintMapping] = &[
    ConstructLintMapping {
        diagnostic: "bpmn.missing_host_task_contract",
        repair: "Keep the multi-instance owner as one serviceTask with qianji prompt, inputs, outputs, and implementation metadata.",
    },
    ConstructLintMapping {
        diagnostic: "bpmn.unsupported_loop_configuration",
        repair: "Use one bounded parallel multiInstanceLoopCharacteristics block with omitted or isSequential=\"false\". Choose exactly one expansion mode: integer loopCardinality, or collection-backed loopDataInputRef plus inputDataItem. If aggregating per-iteration output, provide both loopDataOutputRef and outputDataItem and keep the output path different from the input path.",
    },
];

const INTERACTIVE_LOOP_LINT: &[ConstructLintMapping] = &[
    ConstructLintMapping {
        diagnostic: "bpmn.loop_risk.unbounded_control_cycle",
        repair: "Feed the prior userTask answer into an in-cycle serviceTask, have that task emit every loop gateway variable, and ensure each revisit changes declared userTask inputs such as currentQuestion, currentChoices, attempt, or questionsRemaining.",
    },
    ConstructLintMapping {
        diagnostic: "pi-wendao.runtime.user_prompt_stall",
        repair: "Do not return to the same userTask with unchanged resolved question, choices, and qianji:inputs. Insert or repair an upstream serviceTask that consumes the prior answer and emits changed progress state before the next user prompt.",
    },
    ConstructLintMapping {
        diagnostic: "pi-wendao.runtime.invalid_dynamic_choices",
        repair: "Use producer XML <qianji:outputSchema name=\"currentChoices\" kind=\"choice_array\" value=\"required\" label=\"optional\" description=\"optional\"/> with consumer XML <qianji:choices ref=\"currentChoices\"/>.",
    },
];

const CONSTRUCT_CARDS: &[ConstructCard] = &[
    agent::card(TASK_CONFIG_LINT),
    multi_instance::card(PARALLEL_MULTI_INSTANCE_LINT),
    interaction::card(TASK_CONFIG_LINT),
    loop_progress::card(INTERACTIVE_LOOP_LINT),
    gateway::card(GATEWAY_LINT),
    dmn::card(DMN_LINT),
];

/// Return all registered construct cards in deterministic index order.
#[must_use]
pub(crate) fn construct_cards() -> &'static [ConstructCard] {
    CONSTRUCT_CARDS
}

/// Find a construct card by stable id.
#[must_use]
pub(crate) fn find_construct_card(id: &str) -> Option<&'static ConstructCard> {
    CONSTRUCT_CARDS.iter().find(|card| card.id == id)
}

/// Return compact index entries in deterministic order.
#[must_use]
pub(crate) fn construct_index_entries(cards: &[ConstructCard]) -> Vec<ConstructIndexEntry> {
    cards
        .iter()
        .map(|card| ConstructIndexEntry {
            id: card.id,
            domain: card.domain,
            status: card.status,
            summary: card.summary,
        })
        .collect()
}
