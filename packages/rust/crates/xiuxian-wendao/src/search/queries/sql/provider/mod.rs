//! `search::queries::sql::provider` owns Wendao queries sql provider behavior.

#[path = "metadata.rs"]
pub(crate) mod metadata;
#[cfg(feature = "runtime-transport")]
#[path = "route.rs"]
mod route;

#[cfg(feature = "runtime-transport")]
pub use self::route::StudioSqlFlightRouteProvider;
