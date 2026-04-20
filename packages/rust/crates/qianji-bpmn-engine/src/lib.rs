//! Standalone BPMN engine for Qianji workflow integration.
//!
//! The current implementation covers a bounded BPMN parser/IR subset, a bounded
//! runtime kernel, sequence-guarded JSON checkpoint persistence in Valkey,
//! one feature-gated local SQL checkpoint path for lightweight client-side
//! storage, explicit host/wait boundary request-resume seams plus lease-key
//! ownership for distributed checkpoint writers, deterministic frontier
//! snapshots plus explicit frontier proposal/reduction and deterministic batch
//! execution seams for multi-token runtime planning, and a crate-owned bounded
//! DMN parse and evaluation contract plus LLM-friendly BPMN/DMN lint reports.
//! Parser-owned bundle snapshots can now also attach bounded DMN sources to one
//! BPMN package so local business-rule execution is populated from parse-time
//! inputs instead of test-only manual wiring.
//! Bounded `parallelGateway` split/join semantics and deterministic
//! `exclusiveGateway` pass-through routing plus one bounded exclusive
//! `eventBasedGateway` whose outgoing targets are message/signal/timer
//! `intermediateCatchEvent` waits, plus `intermediateCatchEvent` waits backed
//! by `messageEventDefinition`, `signalEventDefinition`, and snapshot-style
//! `timerEventDefinition`, plus one interrupting timer `boundaryEvent` on one
//! host-blocking task, plus one bounded `callActivity` that targets another
//! process in the same BPMN package, plus bounded `standardLoopCharacteristics`
//! on one serviceTask, userTask, manualTask, or businessRuleTask, plus bounded
//! sequential `multiInstanceLoopCharacteristics isSequential="true"` with
//! integer `loopCardinality` on those same host-blocking task kinds are
//! supported. The bounded DMN evaluator also supports wildcard matching,
//! literal equality, numeric unary comparisons, bounded numeric ranges,
//! ISO date literals, ISO date comparisons, and bounded ISO date ranges.
//! BPMN `businessRuleTask` can also execute locally when the package carries a
//! matching engine-owned DMN decision definition; otherwise it falls back to
//! the existing host seam. Inclusive gateways, embedded `subProcess` bodies,
//! non-interrupting boundaries, parallel multi-instance expansion,
//! multi-instance data bindings, completion conditions, full timer execution
//! semantics, parser-owned BPMN+DMN bundle ingestion, date-time/function FEEL
//! behavior, and richer orchestration slices remain deferred.

mod checkpoint;
mod dmn;
mod error;
mod host;
mod ir;
mod lint;
mod parser;
mod runtime;

pub use checkpoint::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnCheckpointEnvelope, decode_checkpoint_json,
    encode_checkpoint_json, lease_key, state_key,
};
#[cfg(feature = "valkey")]
pub use checkpoint::{
    delete_checkpoint, delete_checkpoint_as_owner, load_checkpoint, release_checkpoint_lease,
    renew_checkpoint_lease, save_checkpoint, save_checkpoint_as_owner,
    try_acquire_checkpoint_lease,
};
#[cfg(feature = "sqlite")]
pub use checkpoint::{delete_checkpoint_sql, load_checkpoint_sql, save_checkpoint_sql};
pub use dmn::{
    DmnBindingKind, DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateRangeBound,
    DmnDecisionDefinition, DmnDecisionRef, DmnDecisionTable, DmnEvaluationRequest,
    DmnEvaluationResult, DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnNumericComparison,
    DmnNumericRange, DmnNumericRangeBound, DmnOutputClause, DmnOutputEntry, DmnRule, DmnSourceFile,
    evaluate_dmn_decision, parse_dmn_decision,
};
pub use error::BpmnEngineError;
pub use host::{
    BpmnHostBridge, BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome,
    EventPollRequest, HostBridgeError, ManualTaskOutcome, ManualTaskRequest,
    PendingHostWorkRequest, PendingHostWorkResult, RepeatExecutionContext,
    SequentialMultiInstanceContext, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome,
    UserTaskRequest,
};
pub use ir::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind, BpmnIndexRange, BpmnNodeIndex,
    BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec, BpmnTimerKind, BpmnTimerSpec,
    ProcessKey,
};
pub use lint::{
    LintDomain, LintIssue, LintReport, LintSeverity, lint_bpmn_source, lint_dmn_source,
};
pub use parser::{
    BpmnBundleSnapshot, BpmnParseOptions, BpmnSourceFile, parse_bpmn_bundle, parse_bpmn_package,
};
pub use runtime::{
    BpmnAdvanceOutcome, BpmnFrontierEntry, BpmnFrontierEntryStatus, BpmnFrontierExecutionBatch,
    BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep, BpmnFrontierParallelJoinMerge,
    BpmnFrontierPlan, BpmnFrontierPlanAction, BpmnFrontierProposalSet, BpmnFrontierSnapshot,
    BpmnInstanceInit, BpmnInstanceState, CallActivityFrame, EventCompetitionState,
    InstanceLifecycle, JoinRuntimeState, NodeRuntimeState, NodeRuntimeStatus, PendingHostWork,
    PendingHostWorkKind, SequentialMultiInstanceState, StandardLoopState, SuspendReason,
    TokenRecord, WaitKind, WaitRegistration, advance_instance, apply_event_poll_outcome,
    apply_pending_host_work_result, build_event_poll_request, build_pending_host_work_request,
    build_pending_host_work_requests, collect_frontier_proposals, create_instance,
    merge_frontier_execution_steps, plan_frontier_step, reduce_frontier_plan, snapshot_frontier,
};

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");
