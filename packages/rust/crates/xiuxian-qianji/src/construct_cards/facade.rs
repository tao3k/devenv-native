use super::{ConstructCard, ConstructIndexEntry, catalog, render};

/// Return all registered construct cards in deterministic index order.
#[must_use]
pub fn construct_cards() -> &'static [ConstructCard] {
    catalog::construct_cards()
}

/// Find a construct card by stable id.
#[must_use]
pub fn find_construct_card(id: &str) -> Option<&'static ConstructCard> {
    catalog::find_construct_card(id)
}

/// Return compact index entries in deterministic order.
#[must_use]
pub fn construct_index_entries(cards: &[ConstructCard]) -> Vec<ConstructIndexEntry> {
    catalog::construct_index_entries(cards)
}

/// Render a compact construct-card table of contents.
#[must_use]
pub fn render_construct_index(cards: &[ConstructCard]) -> String {
    render::render_construct_index(cards)
}

/// Render the construct index as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static catalog cannot be serialized.
pub fn render_construct_index_json(cards: &[ConstructCard]) -> serde_json::Result<String> {
    render::render_construct_index_json(cards)
}

/// Render one detailed construct card.
#[must_use]
pub fn render_construct_card(card: &ConstructCard) -> String {
    render::render_construct_card(card)
}

/// Render one detailed construct card as pretty JSON.
///
/// # Errors
///
/// Returns an error if the static construct card cannot be serialized.
pub fn render_construct_card_json(card: &ConstructCard) -> serde_json::Result<String> {
    render::render_construct_card_json(card)
}
