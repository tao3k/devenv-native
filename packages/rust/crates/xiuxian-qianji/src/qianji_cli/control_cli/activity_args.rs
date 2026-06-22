//! Shared activity command argument contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivitySettleOutcomeArg {
    Complete,
    Fail,
}
