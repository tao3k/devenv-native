//! Named identifier surfaces for Plugin Arrow exchange helpers.

/// Borrowed provider identifier used when building Plugin Arrow trace ids.
pub type PluginArrowProviderIdRef<'a> = &'a str;
/// Borrowed trace identifier attached to Plugin Arrow request metadata.
pub type PluginArrowTraceIdRef<'a> = &'a str;
