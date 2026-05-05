//! Public observability event iterator surface.

use super::session_events;

/// Iterate all canonical observability event identifiers.
pub fn session_event_ids() -> impl Iterator<Item = &'static str> {
    session_events::SessionEvent::ALL
        .iter()
        .copied()
        .map(session_events::SessionEvent::as_str)
}
