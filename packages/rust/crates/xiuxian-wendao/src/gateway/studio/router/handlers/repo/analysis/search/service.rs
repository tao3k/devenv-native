#[path = "service/execution.rs"]
mod execution;
#[path = "service/imports.rs"]
pub(crate) mod imports;
#[path = "service/typed.rs"]
pub(crate) mod typed;

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/service.rs"]
mod tests;
