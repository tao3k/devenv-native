//! Named identifier surfaces for public query-contract helpers.

/// Borrowed entity identifier accepted by query-contract validators.
pub type EntityIdRef<'a> = &'a str;
/// Borrowed module identifier accepted by query-contract validators.
pub type ModuleIdRef<'a> = &'a str;
/// Borrowed graph node identifier accepted by query-contract validators.
pub type NodeIdRef<'a> = &'a str;
/// Borrowed page identifier accepted by query-contract validators.
pub type PageIdRef<'a> = &'a str;
/// Borrowed repository identifier accepted by query-contract validators.
pub type RepoIdRef<'a> = &'a str;
/// Borrowed request identifier accepted by query-contract validators.
pub type RequestIdRef<'a> = &'a str;
