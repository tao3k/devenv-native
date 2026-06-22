//! Contract-feedback CLI feature folder.
//!
//! Start with `api`; it is the single visible entry seam for this folder.

mod api;
mod config;
mod execute;
mod output;
mod parse;
mod support;
mod types;

#[cfg(test)]
pub(crate) use api::{
    ContractFeedbackCliCommand, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, REST_DOCS_PACK_ID,
    RestDocsCliCommand, build_contract_feedback_config,
    run_deterministic_rest_docs_contract_feedback, run_scaffold_rest_docs_contract_feedback,
};
pub(crate) use api::{handle_contract_feedback_command, parse_contract_feedback_command};
