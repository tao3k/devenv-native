#[path = "provider/metadata.rs"]
pub(crate) mod metadata;
#[cfg(feature = "runtime-transport")]
#[path = "provider/route.rs"]
mod route;

#[cfg(feature = "runtime-transport")]
pub(crate) use self::route::StudioSqlFlightRouteProvider;
