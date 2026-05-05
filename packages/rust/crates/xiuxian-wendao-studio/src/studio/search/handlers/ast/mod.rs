mod batch;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/search/handlers/ast/http.rs"]
mod http;
mod provider;
mod response;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/search/handlers/ast/mod.rs"]
mod tests;

#[cfg(test)]
pub use http::search_ast;
pub(crate) use provider::StudioAstSearchFlightRouteProvider;
