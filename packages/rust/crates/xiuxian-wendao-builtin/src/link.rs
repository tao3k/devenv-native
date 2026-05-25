// Keep builtin plugin crates linked so their `inventory`-submitted registrars
// remain visible to the bootstrap bundle without widening any host-facing API.
use xiuxian_julia_core as _;

/// Ensure builtin plugin crates stay linked into the current host build.
#[inline]
pub(crate) fn ensure_builtin_plugins_linked() {}
