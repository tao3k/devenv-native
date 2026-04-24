//! Standalone BPMN engine for Qianji workflow integration.
//!
//! The current implementation covers a bounded BPMN parser/IR subset, a bounded
//! runtime kernel, sequence-guarded JSON checkpoint persistence in Valkey,
//! one feature-gated local SQL checkpoint path for lightweight client-side
//! storage, explicit host/wait boundary request-resume seams plus lease-key
//! ownership for distributed checkpoint writers, deterministic frontier
//! snapshots plus explicit frontier proposal/reduction and deterministic batch
//! execution seams for multi-token runtime planning, and a crate-owned bounded
//! DMN parse and evaluation contract plus one non-executable DMN document
//! snapshot surface and LLM-friendly BPMN/DMN lint reports. Parser-owned
//! bundle snapshots can now also attach bounded DMN sources to one BPMN
//! package, including multiple bounded decisions plus one bounded `inputData`
//! registry and one bounded top-level `businessKnowledgeModel` registry from
//! one DMN source, so local business-rule execution, bounded same-source input
//! aliasing, and later same-source knowledge lookup prerequisites are
//! populated from parse-time inputs instead of test-only manual wiring.
//! Bounded `parallelGateway` split/join semantics, bounded
//! `exclusiveGateway` routing with simple boolean-path or numeric-comparison
//! outgoing `sequenceFlow` `conditionExpression` values plus one optional
//! `default` flow, one bounded structured `inclusiveGateway` subset with the
//! same condition/default routing rules plus one matching linear join
//! fragment,
//! and one bounded exclusive `eventBasedGateway` whose outgoing targets are
//! message/signal/timer `intermediateCatchEvent` waits, plus
//! `intermediateCatchEvent` waits backed by `messageEventDefinition`,
//! `signalEventDefinition`, and snapshot-style `timerEventDefinition`, plus
//! one interrupting timer, message, or signal `boundaryEvent` on one
//! host-blocking task, plus one non-interrupting timer, message, or signal
//! `boundaryEvent` on one non-repeating or bounded
//! `standardLoopCharacteristics`, sequential multi-instance, or parallel
//! multi-instance host-blocking task, plus
//! one bounded embedded `subProcess` body with
//! exactly one nested `startEvent` and at least one nested `endEvent`, plus
//! one bounded embedded subprocess owner that may expose one interrupting
//! timer, message, or signal `boundaryEvent` plus one or more interrupting
//! error `boundaryEvent` nodes on that same owner, where the interrupting
//! parent timer/message/signal boundary may cancel the child shell before
//! restoring the parent frame, one or more nested error ends may each
//! restore the parent frame, preserve variable mutations, and route through
//! every matching parent error boundary including one catch-all boundary,
//! while normal completion and either supported interrupting winner cancel
//! the non-selected sibling boundaries, plus
//! one bounded `<transaction>` shell with exactly one nested `startEvent` and
//! at least one nested `endEvent`, plus one bounded transaction cancel path
//! with one interrupting cancel `boundaryEvent` attached to that
//! `<transaction>` shell and one nested cancel end that restores the parent
//! frame, rolls back transaction-local variable mutations, and routes through
//! the parent cancel boundary, plus one bounded transaction owner that may
//! expose one interrupting timer, message, or signal `boundaryEvent` plus one
//! interrupting cancel `boundaryEvent`, plus one or more interrupting error
//! `boundaryEvent` nodes, or both cancel and error boundaries adjacent to
//! that same interrupting timer/message/signal boundary, where one or more
//! nested error ends may each restore the parent frame, preserve
//! transaction-local variable mutations, and route through every matching
//! parent error boundary including one catch-all boundary, while normal
//! completion, interrupting external wins, cancel routing, and error routing
//! cancel the non-selected sibling boundaries, and the bounded subset still
//! permits only one interrupting timer/message/signal boundary and one
//! interrupting cancel boundary on that same owner, plus one bounded
//! transaction cancel
//! compensation subset where compensable activities may bind one explicit
//! compensation handler and cancel routing replays those handlers in reverse
//! completion order before the parent cancel boundary fires, plus one
//! throw-compensation end-event subset inside that same transaction shell
//! where one nested end event either uses explicit `activityRef` to replay
//! one already compensable activity or omits `activityRef` to replay every
//! already compensable activity in reverse completion order before the shell
//! completes, and the bounded end-event subset may stay synchronous or set
//! `waitForCompletion="false"` so the parent scope resumes while the
//! compensation queue drains, plus one synchronous or asynchronous
//! throw-compensation intermediate-event subset inside that same transaction
//! shell where one nested intermediate throw event either uses explicit
//! `activityRef` to replay one already compensable activity or omits
//! `activityRef` to replay every already compensable activity in reverse
//! completion order before normal sequence-flow routing resumes, and the
//! asynchronous bounded subset may set `waitForCompletion="false"` so the
//! compensation queue drains while downstream routing continues,
//! plus one bounded `callActivity` that targets another process in the same
//! BPMN package, and one bounded same-package `callActivity` owner may expose
//! one interrupting timer, message, or signal `boundaryEvent` plus one or
//! more interrupting error `boundaryEvent` nodes on that same owner, where
//! the interrupting parent timer/message/signal boundary may cancel the
//! called child process before restoring the parent frame, one or more child
//! error ends may each restore the parent frame, preserve variable
//! mutations, and route through every matching parent error boundary
//! including one catch-all boundary, while normal completion and either
//! supported interrupting winner cancel the non-selected sibling boundaries,
//! plus bounded `standardLoopCharacteristics` on one serviceTask,
//! userTask, manualTask, or businessRuleTask, plus bounded
//! sequential `multiInstanceLoopCharacteristics isSequential="true"` plus
//! bounded parallel `multiInstanceLoopCharacteristics` with omitted or
//! `isSequential="false"` and integer `loopCardinality` on those same
//! host-blocking task kinds, plus one bounded multi-instance
//! `completionCondition` subset over simple boolean variable paths or bounded
//! counter comparisons, plus one bounded collection-backed data-binding subset
//! using `loopDataInputRef`, `inputDataItem`, optional `loopDataOutputRef`,
//! and `outputDataItem`, are supported. The bounded DMN evaluator also
//! supports wildcard matching,
//! literal equality, numeric unary comparisons, bounded numeric ranges,
//! ISO date literals, ISO date comparisons, bounded ISO date ranges, ISO local
//! and RFC3339 offset-aware datetime literals plus one bounded UTC
//! normalization rule for mixed datetime literal equality, comparisons, and
//! ranges, plus signed ISO 8601 day-time and year-month duration literals,
//! comparisons, and bounded ranges, plus ISO time literals, ISO time
//! comparisons, bounded ISO time ranges, and bounded day-time duration
//! fractions such as `duration("P1.5D")`, `duration("P1,5D")`,
//! `duration("PT1.5H")`, `duration("PT1,5H")`, `duration("PT1.5M")`,
//! `duration("PT1,5M")`, `duration("PT1.5S")`, and `duration("PT1,5S")`.
//! BPMN `businessRuleTask` can also execute locally when the package carries a
//! matching engine-owned DMN decision definition; the bounded local DMN path
//! now also includes one direct invocation seam whose invoked text resolves to
//! exactly one same-source top-level `businessKnowledgeModel` by id or
//! invocable `variable` name, whose direct bindings expose simple named
//! parameters plus supported literal-expression arguments, whose target
//! `encapsulatedLogic` provides one supported direct literal-expression body,
//! and whose target must match any direct same-source `requiredKnowledge`
//! declarations preserved on the executable decision; otherwise it falls back
//! to the existing host seam. Broader unstructured
//! inclusive gateways, recursive call chains, broader mixed boundary families
//! on same-package
//! `callActivity` owners or embedded subprocess owners beyond one
//! interrupting timer/message/signal boundary plus one or more interrupting
//! error boundaries, broader transaction-shell boundary families that exceed
//! one interrupting timer/message/signal boundary, exceed one interrupting
//! cancel boundary, or otherwise exceed the bounded same-owner
//! external-plus-cancel-plus-error subset, broader non-interrupting boundary
//! families on subprocess-like owners, full timer execution semantics,
//! compensation event subprocesses, broader throw-compensation forms, more than one cancel
//! boundary on the same transaction owner, broader
//! error propagation beyond those bounded transaction and embedded-subprocess
//! shells,
//! broader `requiredKnowledge` execution, broader business-knowledge-model or
//! decision-service invocation semantics, broader FEEL or script-backed
//! gateway conditions, trailing
//! lower-unit fractional duration handling such as `duration("PT1.5H30S")`,
//! mixed-family duration handling, fractional year-month duration handling
//! such as `duration("P1.5Y")`, broader timezone/function FEEL behavior, and
//! richer orchestration slices remain deferred.

mod bpmn_model_api;
mod bpmn_parse_api;
mod bpmn_snapshot;
mod bpmn_snapshot_api;
mod checkpoint;
mod checkpoint_api;
mod dmn;
mod dmn_api;
mod dmn_duration;
mod dmn_evaluate_api;
mod dmn_model_api;
mod dmn_model_business_knowledge;
mod dmn_model_clause;
mod dmn_model_decision;
mod dmn_model_decision_service;
mod dmn_model_document;
mod dmn_model_input_data;
mod dmn_model_predicate;
mod dmn_model_reference;
mod dmn_parse_api;
mod dmn_snapshot_api;
mod error;
mod host_bridge_api;
mod host_types_api;
mod ir;
mod ir_edge_api;
mod ir_event_api;
mod ir_index_api;
mod ir_node_api;
mod ir_package_api;
mod ir_process_compensation;
mod ir_process_key;
mod ir_process_lookup;
mod ir_process_spec;
mod ir_repeat_api;
mod lint;
mod lint_api;
mod parser;
mod repeat_condition;
mod runtime;
mod runtime_advance_api;
mod runtime_dispatch_api;
mod runtime_frontier_api;
mod runtime_host_dispatch_api;
mod runtime_instance_api;
mod runtime_join_api;
mod runtime_repeat_api;
mod runtime_resume_api;
mod runtime_token_api;
mod runtime_wait_api;

pub use bpmn_model_api::{
    BpmnCollaborationSnapshot, BpmnDataAssociationSnapshot, BpmnDataInputOutputSnapshot,
    BpmnDataObjectReferenceSnapshot, BpmnDataObjectSnapshot, BpmnDataStoreReferenceSnapshot,
    BpmnDataStoreSnapshot, BpmnDocumentSnapshot, BpmnIoSpecificationSnapshot, BpmnLaneSetSnapshot,
    BpmnLaneSnapshot, BpmnMessageFlowSnapshot, BpmnParticipantSnapshot, BpmnProcessSnapshot,
    BpmnRootSnapshot,
};
pub use bpmn_parse_api::{
    BpmnBundleSnapshot, BpmnParseOptions, BpmnSourceFile, parse_bpmn_bundle, parse_bpmn_package,
};
pub use bpmn_snapshot_api::snapshot_bpmn_source;
pub use checkpoint_api::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnCheckpointEnvelope, decode_checkpoint_json,
    encode_checkpoint_json, lease_key, state_key,
};
#[cfg(feature = "valkey")]
pub use checkpoint_api::{
    delete_checkpoint, delete_checkpoint_as_owner, load_checkpoint, release_checkpoint_lease,
    renew_checkpoint_lease, save_checkpoint, save_checkpoint_as_owner,
    try_acquire_checkpoint_lease,
};
#[cfg(feature = "sqlite")]
pub use checkpoint_api::{delete_checkpoint_sql, load_checkpoint_sql, save_checkpoint_sql};
pub use dmn_api::{
    DmnAssociationSnapshot, DmnBindingKind, DmnBoundsSnapshot, DmnBusinessKnowledgeModelDefinition,
    DmnBusinessKnowledgeModelSnapshot, DmnComparisonOperator, DmnContextEntry,
    DmnContextExpression, DmnDateComparison, DmnDateRange, DmnDateRangeBound,
    DmnDateTimeComparison, DmnDateTimeRange, DmnDateTimeRangeBound, DmnDecisionDefinition,
    DmnDecisionRef, DmnDecisionServiceDefinition, DmnDecisionServiceDividerLineSnapshot,
    DmnDecisionServiceReference, DmnDecisionServiceSnapshot, DmnDecisionSnapshot, DmnDecisionTable,
    DmnDiagramSnapshot, DmnDmndiSnapshot, DmnDocumentSnapshot, DmnDurationComparison,
    DmnDurationRange, DmnDurationRangeBound, DmnEdgeSnapshot, DmnElementCollectionSnapshot,
    DmnEvaluationRequest, DmnEvaluationResult, DmnGroupSnapshot, DmnHitPolicy,
    DmnInformationRequirementReference, DmnInputClause, DmnInputDataDefinition,
    DmnInputDataSnapshot, DmnInputEntry, DmnInvocation, DmnInvocationBinding,
    DmnInvocationParameter, DmnItemComponentSnapshot, DmnItemDefinitionSnapshot,
    DmnKnowledgeRequirementReference, DmnKnowledgeSourceSnapshot, DmnLabelSnapshot,
    DmnListExpression, DmnLiteralExpression, DmnNumericComparison, DmnNumericRange,
    DmnNumericRangeBound, DmnOrganizationUnitSnapshot, DmnOutputClause, DmnOutputEntry,
    DmnPerformanceIndicatorSnapshot, DmnRelationColumn, DmnRelationExpression, DmnRelationRow,
    DmnRootSnapshot, DmnRule, DmnShapeSnapshot, DmnSourceFile, DmnTextAnnotationSnapshot,
    DmnTimeComparison, DmnTimeRange, DmnTimeRangeBound, DmnVariableSnapshot, DmnWaypointSnapshot,
    evaluate_dmn_decision, parse_dmn_decision, parse_dmn_decisions, snapshot_dmn_source,
};
pub use error::BpmnEngineError;
pub use host_bridge_api::BpmnHostBridge;
pub use host_types_api::{
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ParallelMultiInstanceContext,
    PendingHostWorkRequest, PendingHostWorkResult, RepeatExecutionContext, ScriptTaskOutcome,
    ScriptTaskRequest, SendTaskOutcome, SendTaskRequest, SequentialMultiInstanceContext,
    ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome, UserTaskRequest,
};
pub use ir_edge_api::BpmnEdgeSpec;
pub use ir_event_api::{BpmnEventKind, BpmnEventSpec, BpmnTimerKind, BpmnTimerSpec};
pub use ir_index_api::{BpmnIndexRange, BpmnNodeIndex};
pub use ir_node_api::{
    BpmnGatewayKind, BpmnNodeKind, BpmnNodeSpec, BpmnScriptTaskSpec, BpmnSubProcessKind,
};
pub use ir_package_api::BpmnPackage;
pub use ir_process_compensation::BpmnCompensationHandlerSpec;
pub use ir_process_key::ProcessKey;
pub use ir_process_spec::BpmnProcessSpec;
pub use ir_repeat_api::{
    BpmnMultiInstanceDataBindingSpec, BpmnParallelMultiInstanceSpec, BpmnRepeatSpec,
    BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec,
};
pub use lint_api::{
    LintDomain, LintIssue, LintReport, LintSeverity, lint_bpmn_source, lint_dmn_source,
};
pub use runtime_advance_api::{BpmnAdvanceOutcome, advance_instance};
pub use runtime_dispatch_api::{PendingHostWork, PendingHostWorkKind};
pub use runtime_frontier_api::{
    BpmnFrontierEntry, BpmnFrontierEntryStatus, BpmnFrontierExecutionBatch,
    BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep, BpmnFrontierParallelJoinMerge,
    BpmnFrontierPlan, BpmnFrontierPlanAction, BpmnFrontierProposalSet, BpmnFrontierSnapshot,
    collect_frontier_proposals, merge_frontier_execution_steps, plan_frontier_step,
    reduce_frontier_plan, snapshot_frontier,
};
pub use runtime_host_dispatch_api::{
    build_pending_host_work_request, build_pending_host_work_requests,
};
pub use runtime_instance_api::{
    BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnInstanceInit, BpmnInstanceState,
    CallActivityFrame, EventCompetitionState, InstanceLifecycle, NodeRuntimeState,
    NodeRuntimeStatus, SuspendReason, create_instance,
};
pub use runtime_join_api::JoinRuntimeState;
pub use runtime_repeat_api::{
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
    ParallelMultiInstanceIterationState, ParallelMultiInstanceState, SequentialMultiInstanceState,
    StandardLoopState,
};
pub use runtime_resume_api::apply_pending_host_work_result;
pub use runtime_token_api::{InclusiveJoinHint, TokenRecord};
pub use runtime_wait_api::{
    WaitKind, WaitRegistration, apply_event_poll_outcome, build_event_poll_request,
};

xiuxian_testing::crate_testing_source_gate!("../tests/unit/lib_policy.rs");
