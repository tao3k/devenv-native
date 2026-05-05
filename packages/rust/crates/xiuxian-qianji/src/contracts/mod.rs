//! Contract feature folder.
//!
//! Start with `api`; it is the single visible entry seam for this folder.

mod api;
#[path = "../contracts_bindings.rs"]
mod bindings;
#[path = "../contracts_execution.rs"]
mod execution;
#[path = "../contracts_flowhub_contract.rs"]
mod flowhub_contract;
#[path = "../contracts_flowhub_grammar.rs"]
mod flowhub_grammar;
#[path = "../contracts_flowhub_manifest.rs"]
mod flowhub_manifest;
#[path = "../contracts_flowhub_root.rs"]
mod flowhub_root;
#[path = "../contracts_flowhub_validation.rs"]
mod flowhub_validation;
#[path = "../contracts_manifest.rs"]
mod manifest;
#[path = "../contracts_mechanism.rs"]
mod mechanism;
#[path = "../contracts_wendao_docs/mod.rs"]
mod wendao_docs;
#[path = "../contracts_workdir_manifest.rs"]
mod workdir_manifest;

pub use api::{
    EdgeDefinition, FlowInstruction, FlowhubGraphContract, FlowhubGraphNodeContract,
    FlowhubGraphSurfaceContract, FlowhubGraphTopology, FlowhubGraphWorkdirContract,
    FlowhubModuleExports, FlowhubModuleManifest, FlowhubModuleMetadata, FlowhubRootManifest,
    FlowhubRootMetadata, FlowhubScenarioManifest, FlowhubScenarioPlanning, FlowhubScenarioTemplate,
    FlowhubStructureContract, FlowhubTemplateComposition, FlowhubValidationKind,
    FlowhubValidationRule, FlowhubValidationScope, NodeDefinition, NodeLlmBinding,
    NodeQianhuanBinding, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest, QianjiMechanism,
    QianjiOutput, TemplateLinkRef, TemplateLinkSpec, TemplateUseSpec, WendaoDocsContractShow,
    WorkdirCheck, WorkdirManifest, WorkdirPlan, render_wendao_docs_contract_show,
    show_wendao_docs_contract,
};
pub(crate) use wendao_docs::{load_wendao_docs_contract, validate_cli_call, validate_http_call};
