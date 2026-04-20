//! Error types for the BPMN engine scaffold.

/// Result alias for BPMN engine operations.
pub type Result<T> = std::result::Result<T, BpmnEngineError>;

/// Error surface for scaffold and future engine operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum BpmnEngineError {
    /// Returned by public API shells that are intentionally not implemented yet.
    #[error("operation '{operation}' is not implemented in the current scaffold slice")]
    UnsupportedOperation {
        /// The public operation name.
        operation: &'static str,
    },
    /// Returned when a requested process does not exist in the package.
    #[error("process '{process_id}' was not found in the BPMN package")]
    MissingProcess {
        /// The requested process identifier.
        process_id: String,
    },
    /// Returned when BPMN parsing receives an unsupported source bundle shape.
    #[error("unsupported BPMN source bundle: expected exactly one source file, got {count}")]
    UnsupportedSourceBundle {
        /// Provided source-file count.
        count: usize,
    },
    /// Returned when one BPMN source cannot be parsed as XML.
    #[error("invalid BPMN XML in source '{source_id}': {message}")]
    InvalidXml {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// XML parser diagnostic.
        message: String,
    },
    /// Returned when the BPMN document has no root element.
    #[error("BPMN source '{source_id}' has no root XML element")]
    MissingRootElement {
        /// Source identifier used for diagnostics.
        source_id: String,
    },
    /// Returned when a required BPMN attribute is missing.
    #[error(
        "BPMN source '{source_id}' element '{element}' is missing required attribute '{attribute}'"
    )]
    MissingAttribute {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Element local name.
        element: String,
        /// Missing attribute local name.
        attribute: String,
    },
    /// Returned when the bounded slice encounters an unsupported BPMN element.
    #[error(
        "BPMN source '{source_id}' uses unsupported element '{element}' in process '{process_id}'"
    )]
    UnsupportedElement {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Process identifier where the unsupported element appeared.
        process_id: String,
        /// Unsupported element local name.
        element: String,
    },
    /// Returned when one DMN source cannot be parsed as XML.
    #[error("invalid DMN XML in source '{source_id}': {message}")]
    InvalidDmnXml {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// XML parser diagnostic.
        message: String,
    },
    /// Returned when the DMN document has no root element.
    #[error("DMN source '{source_id}' has no root XML element")]
    MissingDmnRootElement {
        /// Source identifier used for diagnostics.
        source_id: String,
    },
    /// Returned when a required DMN attribute is missing.
    #[error(
        "DMN source '{source_id}' element '{element}' is missing required attribute '{attribute}'"
    )]
    MissingDmnAttribute {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Element local name.
        element: String,
        /// Missing attribute local name.
        attribute: String,
    },
    /// Returned when a DMN document does not contain a decision.
    #[error("DMN source '{source_id}' does not contain any decisions")]
    MissingDmnDecision {
        /// Source identifier used for diagnostics.
        source_id: String,
    },
    /// Returned when more than one DMN decision appears in one bounded source.
    #[error(
        "DMN source '{source_id}' contains unsupported decision count {count}; expected exactly 1"
    )]
    UnsupportedDmnDecisionCount {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Observed decision count.
        count: usize,
    },
    /// Returned when a DMN decision does not contain a decision table.
    #[error("DMN decision '{decision_id}' does not contain any decision tables")]
    MissingDmnDecisionTable {
        /// Decision identifier.
        decision_id: String,
    },
    /// Returned when more than one DMN table appears in one bounded decision.
    #[error(
        "DMN decision '{decision_id}' contains unsupported table count {count}; expected exactly 1"
    )]
    UnsupportedDmnDecisionTableCount {
        /// Decision identifier.
        decision_id: String,
        /// Observed table count.
        count: usize,
    },
    /// Returned when one DMN hit policy exceeds the bounded slice.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' uses unsupported hit policy '{hit_policy}'"
    )]
    UnsupportedDmnHitPolicy {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Decision identifier.
        decision_id: String,
        /// Observed hit policy.
        hit_policy: String,
    },
    /// Returned when a DMN literal exceeds the bounded evaluator slice.
    #[error("DMN source '{source_id}' uses unsupported literal expression '{literal}'")]
    UnsupportedDmnLiteral {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Unsupported literal text.
        literal: String,
    },
    /// Returned when a DMN unary test exceeds the bounded evaluator slice.
    #[error("DMN source '{source_id}' uses unsupported unary test '{expression}'")]
    UnsupportedDmnUnaryTest {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Unsupported unary-test expression text.
        expression: String,
    },
    /// Returned when one DMN rule has a different number of entries than the table clauses.
    #[error(
        "DMN source '{source_id}' rule '{rule_id}' has invalid arity: expected {expected_inputs} inputs/{expected_outputs} outputs, got {actual_inputs} inputs/{actual_outputs} outputs"
    )]
    InvalidDmnRuleArity {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Rule identifier.
        rule_id: String,
        /// Expected input count.
        expected_inputs: usize,
        /// Actual input count.
        actual_inputs: usize,
        /// Expected output count.
        expected_outputs: usize,
        /// Actual output count.
        actual_outputs: usize,
    },
    /// Returned when an evaluation request references the wrong DMN decision.
    #[error("DMN evaluation expected decision '{expected}', but got '{actual}'")]
    DmnDecisionMismatch {
        /// Expected decision identifier.
        expected: String,
        /// Actual decision identifier.
        actual: String,
    },
    /// Returned when one package-level DMN registry lookup is ambiguous.
    #[error(
        "DMN decision reference '{decision_id}'{source_suffix} is ambiguous across {count} registered definitions"
    )]
    AmbiguousDmnDecisionReference {
        /// Requested decision identifier.
        decision_id: String,
        /// Optional source identifier on the request.
        source_id: Option<String>,
        /// Number of matching registered definitions.
        count: usize,
        /// Derived suffix for the error display.
        source_suffix: String,
    },
    /// Returned when no process definitions are present in the BPMN package.
    #[error("BPMN source '{source_id}' does not contain any process definitions")]
    MissingProcessDefinitions {
        /// Source identifier used for diagnostics.
        source_id: String,
    },
    /// Returned when duplicate process identifiers appear in one package.
    #[error("duplicate BPMN process identifier '{process_id}' in package '{package_id}'")]
    DuplicateProcessId {
        /// Package identifier.
        package_id: String,
        /// Duplicate process identifier.
        process_id: String,
    },
    /// Returned when duplicate node identifiers appear in one process.
    #[error("duplicate BPMN node identifier '{node_id}' in process '{process_id}'")]
    DuplicateNodeId {
        /// Owning process identifier.
        process_id: String,
        /// Duplicate node identifier.
        node_id: String,
    },
    /// Returned when duplicate sequence-flow identifiers appear in one process.
    #[error("duplicate BPMN sequence flow identifier '{flow_id}' in process '{process_id}'")]
    DuplicateSequenceFlowId {
        /// Owning process identifier.
        process_id: String,
        /// Duplicate sequence-flow identifier.
        flow_id: String,
    },
    /// Returned when a process is structurally invalid for the bounded subset.
    #[error("BPMN process '{process_id}' is missing a required {element}")]
    MissingRequiredProcessElement {
        /// Process identifier.
        process_id: String,
        /// Missing required element kind.
        element: &'static str,
    },
    /// Returned when one BPMN node is missing a required child structure.
    #[error("BPMN process '{process_id}' node '{node_id}' is missing required {element}")]
    MissingRequiredNodeElement {
        /// Process identifier.
        process_id: String,
        /// BPMN node identifier.
        node_id: String,
        /// Missing required element kind.
        element: &'static str,
    },
    /// Returned when a business-rule task has no DMN decision binding.
    #[error(
        "BPMN business rule task '{node_id}' in process '{process_id}' is missing a DMN decision reference"
    )]
    MissingBusinessRuleDecisionRef {
        /// Process identifier.
        process_id: String,
        /// BPMN node identifier.
        node_id: String,
    },
    /// Returned when a sequence flow references a missing source or target.
    #[error(
        "BPMN sequence flow '{flow_id}' in process '{process_id}' references unknown {endpoint} '{node_id}'"
    )]
    UnknownSequenceFlowEndpoint {
        /// Process identifier.
        process_id: String,
        /// Sequence flow identifier.
        flow_id: String,
        /// Endpoint kind.
        endpoint: &'static str,
        /// Missing node identifier.
        node_id: String,
    },
    /// Returned when the bounded event slice encounters multiple event
    /// definitions on one node.
    #[error(
        "BPMN source '{source_id}' node '{node_id}' in process '{process_id}' uses unsupported multiple event definitions"
    )]
    UnsupportedMultipleEventDefinitions {
        /// Source identifier used for diagnostics.
        source_id: String,
        /// Process identifier.
        process_id: String,
        /// BPMN node identifier.
        node_id: String,
    },
    /// Returned when one boundary event references an unknown attached node.
    #[error(
        "BPMN process '{process_id}' boundary event '{node_id}' references unknown attached node '{attached_to_node_id}'"
    )]
    UnknownBoundaryAttachment {
        /// Process identifier.
        process_id: String,
        /// Boundary-event BPMN node identifier.
        node_id: String,
        /// Missing attached target node identifier.
        attached_to_node_id: String,
    },
    /// Returned when one boundary event exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedBoundaryEventConfiguration {
        /// Process identifier.
        process_id: String,
        /// Boundary-event BPMN node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one compensation handler configuration exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' compensation node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedCompensationConfiguration {
        /// Process identifier.
        process_id: String,
        /// Compensation-boundary or compensation-activity BPMN node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one event-based gateway exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' event-based gateway '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedEventBasedGatewayConfiguration {
        /// Process identifier.
        process_id: String,
        /// Event-based gateway BPMN node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one loop configuration exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' loop node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedLoopConfiguration {
        /// Process identifier.
        process_id: String,
        /// BPMN node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one call activity references an unknown process id.
    #[error(
        "BPMN process '{process_id}' call activity '{node_id}' references unknown called process '{called_process_id}'"
    )]
    UnknownCalledProcess {
        /// Owning process identifier.
        process_id: String,
        /// Call-activity BPMN node identifier.
        node_id: String,
        /// Missing called-process identifier.
        called_process_id: String,
    },
    /// Returned when one subprocess or call-activity configuration exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' subprocess node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedSubProcessConfiguration {
        /// Owning process identifier.
        process_id: String,
        /// BPMN subprocess or call-activity node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one transaction-specific cancel path exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' transaction node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedTransactionConfiguration {
        /// Process identifier.
        process_id: String,
        /// BPMN node identifier.
        node_id: String,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when checkpoint JSON encoding or decoding fails.
    #[error("checkpoint codec error: {0}")]
    CheckpointCodec(String),
    /// Returned when host-result application is attempted without pending work.
    #[error("workflow instance '{instance_id}' does not have pending host work")]
    MissingPendingHostWork {
        /// Owning workflow instance identifier.
        instance_id: String,
    },
    /// Returned when one workflow instance has multiple pending host-work
    /// entries but a singleton request surface was used.
    #[error(
        "workflow instance '{instance_id}' has {count} pending host-work entries; a singleton request is ambiguous"
    )]
    AmbiguousPendingHostWork {
        /// Owning workflow instance identifier.
        instance_id: String,
        /// Observed pending-work count.
        count: usize,
    },
    /// Returned when host-result application references an unknown blocked
    /// token.
    #[error(
        "workflow instance '{instance_id}' does not have pending host work for token {token_id}"
    )]
    MissingPendingHostWorkToken {
        /// Owning workflow instance identifier.
        instance_id: String,
        /// Missing blocked token identifier.
        token_id: u64,
    },
    /// Returned when a host result does not match the pending host-work kind.
    #[error(
        "pending host work for node {node_index} expects kind '{expected}', but got '{actual}'"
    )]
    HostResultKindMismatch {
        /// Owning BPMN node index.
        node_index: u32,
        /// Expected pending host-work kind.
        expected: &'static str,
        /// Actual host-result kind.
        actual: &'static str,
    },
    /// Returned when a checkpoint save attempts to overwrite newer state.
    #[error(
        "checkpoint save for instance '{instance_id}' is stale: attempted sequence {attempted_sequence}, stored sequence {stored_sequence}"
    )]
    StaleCheckpointWrite {
        /// Owning workflow instance identifier.
        instance_id: String,
        /// Incoming checkpoint sequence.
        attempted_sequence: u64,
        /// Already stored checkpoint sequence.
        stored_sequence: u64,
    },
    /// Returned when a checkpoint lease TTL is invalid.
    #[error("checkpoint lease ttl must be greater than 0 milliseconds, got {ttl_ms}")]
    InvalidCheckpointLeaseTtl {
        /// Invalid lease TTL in milliseconds.
        ttl_ms: u64,
    },
    /// Returned when an owner-guarded checkpoint write does not hold the lease.
    #[error("workflow instance '{instance_id}' checkpoint lease is not owned by the caller")]
    CheckpointLeaseNotOwned {
        /// Owning workflow instance identifier.
        instance_id: String,
    },
    /// Returned when an event-poll operation is attempted without a wait.
    #[error("workflow instance '{instance_id}' does not have any active wait registrations")]
    MissingWaitRegistration {
        /// Owning workflow instance identifier.
        instance_id: String,
    },
    /// Returned when Valkey checkpoint I/O fails.
    #[error("checkpoint storage operation '{operation}' failed: {message}")]
    CheckpointStorage {
        /// Failing storage operation.
        operation: &'static str,
        /// Backend diagnostic message.
        message: String,
    },
}
