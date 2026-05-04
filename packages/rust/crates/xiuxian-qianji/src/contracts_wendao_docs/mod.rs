//! Wendao docs invocation-contract helpers.

#[path = "contract.rs"]
mod contract;
#[path = "facade.rs"]
mod facade;

pub use facade::{
    WendaoDocsContractShow, render_wendao_docs_contract_show, show_wendao_docs_contract,
};
pub(crate) use facade::{load_wendao_docs_contract, validate_cli_call, validate_http_call};
