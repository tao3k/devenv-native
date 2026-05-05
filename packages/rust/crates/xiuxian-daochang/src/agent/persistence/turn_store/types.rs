//! Shared turn-store persistence payload types.

pub(super) struct TurnStoreOutcome {
    pub(super) label: String,
    pub(super) reward: f32,
}

pub(super) struct StoredTurnEpisode {
    pub(super) id: String,
    pub(super) source: &'static str,
}
