#[path = "service/coverage.rs"]
pub(crate) mod coverage;
#[path = "service/overview.rs"]
pub(crate) mod overview;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/service.rs"]
mod tests;
