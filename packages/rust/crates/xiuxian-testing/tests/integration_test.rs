//! Cargo entry point for xiuxian-testing integration tests.

xiuxian_testing::crate_test_policy_harness!();

#[path = "integration/contracts_kernel.rs"]
mod contracts_kernel;
#[path = "integration/contracts_knowledge_export.rs"]
mod contracts_knowledge_export;
#[path = "integration/contracts_modularity/mod.rs"]
mod contracts_modularity;
#[path = "integration/contracts_rest_docs.rs"]
mod contracts_rest_docs;
#[path = "integration/contracts_runner.rs"]
mod contracts_runner;
#[path = "integration/docs_kernel_contract.rs"]
mod docs_kernel_contract;
