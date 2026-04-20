#[path = "../contract_feedback_pipeline.rs"]
mod pipeline;
#[path = "../contract_feedback_rest_docs.rs"]
mod rest_docs;

pub use pipeline::{
    QianjiContractFeedbackRun, QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
    run_and_persist_contract_feedback_flow, run_contract_feedback_flow,
};
pub use rest_docs::{
    OpenApiFileRestDocsRulePack, build_rest_docs_collection_context,
    build_rest_docs_contract_suite, run_and_persist_rest_docs_contract_feedback,
    run_rest_docs_contract_feedback,
};

#[cfg(feature = "llm")]
pub use pipeline::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};
