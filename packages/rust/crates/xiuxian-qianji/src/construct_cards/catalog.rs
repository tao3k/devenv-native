//! Static construct-card registry split by construct family.
//!
//! Start with `agent` for host-owned service task cards; sibling modules own
//! interaction, gateway, and DMN cards.

use super::{ConstructCard, ConstructIndexEntry, ConstructLintMapping};

mod agent;
mod dmn;
mod gateway;
mod interaction;

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
];

const DMN_LINT: &[ConstructLintMapping] = &[ConstructLintMapping {
    diagnostic: "dmn.invalid_decision_table",
    repair: "Keep one explicit decision id, declared inputs, typed outputs, and rules that match the declared hit policy.",
}];

const CONSTRUCT_CARDS: &[ConstructCard] = &[
    agent::card(TASK_CONFIG_LINT),
    interaction::card(TASK_CONFIG_LINT),
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
