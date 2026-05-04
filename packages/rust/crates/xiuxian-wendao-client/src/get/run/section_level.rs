//! Heading level normalization for local `wendao get` rendering.

pub(crate) fn effective_section_level(level: usize) -> usize {
    level.clamp(1, 6)
}
