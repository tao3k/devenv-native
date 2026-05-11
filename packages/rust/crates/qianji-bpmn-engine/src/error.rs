//! Error types for the BPMN engine scaffold.

/// Result alias for BPMN engine operations.
pub type Result<T> = std::result::Result<T, BpmnEngineError>;

macro_rules! error_string_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized diagnostic value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }
    };
}

error_string_type!(
    /// Identifier value carried by a public BPMN engine diagnostic.
    BpmnErrorId
);
error_string_type!(
    /// Kind discriminator carried by a public BPMN engine diagnostic.
    BpmnErrorKind
);

/// Runtime token identifier carried by a public BPMN engine diagnostic.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct BpmnErrorTokenId(u64);

impl BpmnErrorTokenId {
    /// Returns the raw runtime token identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for BpmnErrorTokenId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.get())
    }
}

impl From<u64> for BpmnErrorTokenId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl PartialEq<u64> for BpmnErrorTokenId {
    fn eq(&self, other: &u64) -> bool {
        self.get() == *other
    }
}

/// Detailed payload for a pending host-work identity mismatch.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error(
    "pending host work identity mismatch for instance '{instance}' token {token}: expected process '{expected_process}' activity '{expected_activity}', got process '{actual_process}' activity '{actual_activity}'"
)]
pub struct BpmnPendingHostWorkIdentityMismatch {
    /// Workflow instance identifier.
    pub instance: String,
    /// Runtime token identifier for the pending host work.
    pub token: u64,
    /// Requested BPMN process identifier.
    pub expected_process: String,
    /// Requested BPMN activity identifier.
    pub expected_activity: String,
    /// Checkpointed BPMN process identifier.
    pub actual_process: String,
    /// Checkpointed BPMN activity identifier.
    pub actual_activity: String,
}

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
        process_id: BpmnErrorId,
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
        source_id: BpmnErrorId,
        /// XML parser diagnostic.
        message: String,
        /// Byte offset reported by the XML reader when available.
        offset: Option<u64>,
    },
    /// Returned when the BPMN document has no root element.
    #[error("BPMN source '{source_id}' has no root XML element")]
    MissingRootElement {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when a required BPMN attribute is missing.
    #[error(
        "BPMN source '{source_id}' element '{element}' is missing required attribute '{attribute}'"
    )]
    MissingAttribute {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
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
        source_id: BpmnErrorId,
        /// Process identifier where the unsupported element appeared.
        process_id: BpmnErrorId,
        /// Unsupported element local name.
        element: String,
    },
    /// Returned when one DMN source cannot be parsed as XML.
    #[error("invalid DMN XML in source '{source_id}': {message}")]
    InvalidDmnXml {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// XML parser diagnostic.
        message: String,
    },
    /// Returned when the DMN document has no root element.
    #[error("DMN source '{source_id}' has no root XML element")]
    MissingDmnRootElement {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when the DMN root element is not `definitions`.
    #[error(
        "DMN source '{source_id}' uses invalid root element '{element}'; expected 'definitions'"
    )]
    UnsupportedDmnRootElement {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Observed root element local name.
        element: String,
    },
    /// Returned when the DMN root does not declare a model namespace.
    #[error("DMN source '{source_id}' is missing a supported DMN model namespace declaration")]
    MissingDmnModelNamespace {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when the DMN root declares a model namespace outside the bounded slice.
    #[error(
        "DMN source '{source_id}' uses unsupported DMN model namespace '{model_namespace_uri}'"
    )]
    UnsupportedDmnModelNamespace {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Observed DMN model namespace URI.
        model_namespace_uri: String,
    },
    /// Returned when the DMN document uses unsupported top-level imports.
    #[error("DMN source '{source_id}' uses unsupported top-level import declarations")]
    UnsupportedDmnImport {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when a required DMN attribute is missing.
    #[error(
        "DMN source '{source_id}' element '{element}' is missing required attribute '{attribute}'"
    )]
    MissingDmnAttribute {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Element local name.
        element: String,
        /// Missing attribute local name.
        attribute: String,
    },
    /// Returned when a DMN document does not contain a decision.
    #[error("DMN source '{source_id}' does not contain any decisions")]
    MissingDmnDecision {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when an exact-one DMN parse entrypoint receives anything other
    /// than exactly one decision.
    #[error(
        "DMN source '{source_id}' contains unsupported decision count {count}; expected exactly 1"
    )]
    UnsupportedDmnDecisionCount {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Observed decision count.
        count: usize,
    },
    /// Returned when a DMN decision does not contain a decision table.
    #[error("DMN decision '{decision_id}' does not contain any decision tables")]
    MissingDmnDecisionTable {
        /// Decision identifier.
        decision_id: BpmnErrorId,
    },
    /// Returned when more than one DMN table appears in one bounded decision.
    #[error(
        "DMN decision '{decision_id}' contains unsupported table count {count}; expected exactly 1"
    )]
    UnsupportedDmnDecisionTableCount {
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Observed table count.
        count: usize,
    },
    /// Returned when one DMN hit policy exceeds the bounded slice.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' uses unsupported hit policy '{hit_policy}'"
    )]
    UnsupportedDmnHitPolicy {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Observed hit policy.
        hit_policy: String,
    },
    /// Returned when a DMN literal exceeds the bounded evaluator slice.
    #[error("DMN source '{source_id}' uses unsupported literal expression '{literal}'")]
    UnsupportedDmnLiteral {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Unsupported literal text.
        literal: String,
    },
    /// Returned when a DMN unary test exceeds the bounded evaluator slice.
    #[error("DMN source '{source_id}' uses unsupported unary test '{expression}'")]
    UnsupportedDmnUnaryTest {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Unsupported unary-test expression text.
        expression: String,
    },
    /// Returned when one DMN rule has a different number of entries than the table clauses.
    #[error(
        "DMN source '{source_id}' rule '{rule_id}' has invalid arity: expected {expected_inputs} inputs/{expected_outputs} outputs, got {actual_inputs} inputs/{actual_outputs} outputs"
    )]
    InvalidDmnRuleArity {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
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
        decision_id: BpmnErrorId,
        /// Optional source identifier on the request.
        source_id: Option<String>,
        /// Number of matching registered definitions.
        count: usize,
        /// Derived suffix for the error display.
        source_suffix: String,
    },
    /// Returned when one business-rule reference resolves ambiguously across
    /// registered local decision services.
    #[error(
        "DMN decision-service reference '{decision_service_id}'{source_suffix} is ambiguous across {count} registered definitions"
    )]
    AmbiguousDmnDecisionServiceReference {
        /// Requested decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Optional source identifier on the request.
        source_id: Option<String>,
        /// Number of matching registered definitions.
        count: usize,
        /// Derived suffix for the error display.
        source_suffix: String,
    },
    /// Returned when one package-level DMN import lookup is ambiguous.
    #[error(
        "DMN import {selector_kind} '{selector_value}' in source '{source_id}' is ambiguous across {count} registered imports"
    )]
    AmbiguousDmnImportReference {
        /// Declaring source identifier used for the lookup.
        source_id: BpmnErrorId,
        /// Selector field used by the lookup.
        selector_kind: &'static str,
        /// Selector value used by the lookup.
        selector_value: String,
        /// Number of matching registered imports.
        count: usize,
    },
    /// Returned when one package-level DMN source-root lookup is ambiguous.
    #[error(
        "DMN source namespace '{namespace}' is ambiguous across {count} registered source roots"
    )]
    AmbiguousDmnSourceNamespace {
        /// Requested DMN business namespace.
        namespace: String,
        /// Number of matching registered source roots.
        count: usize,
    },
    /// Returned when one required-decision href does not stay within the
    /// bounded local-fragment slice.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' uses unsupported information-requirement href '{href}'"
    )]
    UnsupportedDmnInformationRequirementHref {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Observed href value or placeholder.
        href: String,
    },
    /// Returned when one required-decision href points at no registered local
    /// decision in the same source.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' references missing required decision target '{href}'"
    )]
    MissingDmnRequiredDecisionTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Missing local href target.
        href: String,
    },
    /// Returned when one required-input href points at no registered local
    /// input-data definition in the same source.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' references missing required input target '{href}'"
    )]
    MissingDmnRequiredInputTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Missing local href target.
        href: String,
    },
    /// Returned when one required-knowledge href does not stay within the
    /// bounded local-fragment slice.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' uses unsupported knowledge-requirement href '{href}'"
    )]
    UnsupportedDmnKnowledgeRequirementHref {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Observed href value or placeholder.
        href: String,
    },
    /// Returned when one required-knowledge href points at no registered local
    /// business-knowledge-model in the same source.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' references missing required knowledge target '{href}'"
    )]
    MissingDmnRequiredKnowledgeTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Missing local href target.
        href: String,
    },
    /// Returned when one direct invocation target does not resolve to one
    /// same-source registered business-knowledge-model.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' references missing invocation target '{target}'"
    )]
    MissingDmnInvocationTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Missing invocation target text.
        target: String,
    },
    /// Returned when one direct invocation target resolves ambiguously within
    /// the same source.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' references ambiguous invocation target '{target}' across {count} local business-knowledge-model definitions"
    )]
    AmbiguousDmnInvocationTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Ambiguous invocation target text.
        target: String,
        /// Number of matching business-knowledge-model definitions.
        count: usize,
    },
    /// Returned when one direct invocation target is not declared by any
    /// preserved direct same-source required-knowledge edge.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' uses invocation target '{target}' outside its declared required-knowledge contract"
    )]
    UndeclaredDmnInvocationKnowledgeTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
        /// Invocation target text that did not match required knowledge.
        target: String,
    },
    /// Returned when one local decision service exposes no output decisions in
    /// the bounded runtime slice.
    #[error(
        "DMN source '{source_id}' decision service '{decision_service_id}' exposes unsupported output-decision count {count}"
    )]
    UnsupportedDmnDecisionServiceOutputCount {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Number of preserved `outputDecision` references.
        count: usize,
    },
    /// Returned when one decision-service output href does not stay within the
    /// bounded local-fragment slice.
    #[error(
        "DMN source '{source_id}' decision service '{decision_service_id}' uses unsupported output-decision href '{href}'"
    )]
    UnsupportedDmnDecisionServiceOutputHref {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Observed href value or placeholder.
        href: String,
    },
    /// Returned when one decision-service output href points at no registered
    /// local decision in the same source.
    #[error(
        "DMN source '{source_id}' decision service '{decision_service_id}' references missing output decision target '{href}'"
    )]
    MissingDmnDecisionServiceOutputTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Missing local href target.
        href: String,
    },
    /// Returned when one non-output decision-service exposure href does not
    /// stay within the bounded local-fragment slice.
    #[error(
        "DMN source '{source_id}' decision service '{decision_service_id}' uses unsupported {reference_kind} href '{href}'"
    )]
    UnsupportedDmnDecisionServiceReferenceHref {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Direct decision-service child kind.
        reference_kind: BpmnErrorKind,
        /// Observed href value or placeholder.
        href: String,
    },
    /// Returned when one non-output decision-service exposure href points at
    /// no registered local target in the same source.
    #[error(
        "DMN source '{source_id}' decision service '{decision_service_id}' references missing {reference_kind} target '{href}'"
    )]
    MissingDmnDecisionServiceReferenceTarget {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision-service identifier.
        decision_service_id: BpmnErrorId,
        /// Direct decision-service child kind.
        reference_kind: BpmnErrorKind,
        /// Missing local href target.
        href: String,
    },
    /// Returned when bounded local required-decision evaluation encounters a
    /// cycle.
    #[error(
        "DMN source '{source_id}' decision '{decision_id}' participates in a cyclic required-decision dependency"
    )]
    CyclicDmnRequiredDecisionDependency {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Decision identifier.
        decision_id: BpmnErrorId,
    },
    /// Returned when no process definitions are present in the BPMN package.
    #[error("BPMN source '{source_id}' does not contain any process definitions")]
    MissingProcessDefinitions {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
    },
    /// Returned when duplicate process identifiers appear in one package.
    #[error("duplicate BPMN process identifier '{process_id}' in package '{package_id}'")]
    DuplicateProcessId {
        /// Package identifier.
        package_id: BpmnErrorId,
        /// Duplicate process identifier.
        process_id: BpmnErrorId,
    },
    /// Returned when duplicate node identifiers appear in one process.
    #[error("duplicate BPMN node identifier '{node_id}' in process '{process_id}'")]
    DuplicateNodeId {
        /// Owning process identifier.
        process_id: BpmnErrorId,
        /// Duplicate node identifier.
        node_id: BpmnErrorId,
    },
    /// Returned when duplicate sequence-flow identifiers appear in one process.
    #[error("duplicate BPMN sequence flow identifier '{flow_id}' in process '{process_id}'")]
    DuplicateSequenceFlowId {
        /// Owning process identifier.
        process_id: BpmnErrorId,
        /// Duplicate sequence-flow identifier.
        flow_id: BpmnErrorId,
    },
    /// Returned when a process is structurally invalid for the bounded subset.
    #[error("BPMN process '{process_id}' is missing a required {element}")]
    MissingRequiredProcessElement {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Missing required element kind.
        element: &'static str,
    },
    /// Returned when one BPMN node is missing a required child structure.
    #[error("BPMN process '{process_id}' node '{node_id}' is missing required {element}")]
    MissingRequiredNodeElement {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN node identifier.
        node_id: BpmnErrorId,
        /// Missing required element kind.
        element: &'static str,
    },
    /// Returned when a business-rule task has no DMN decision binding.
    #[error(
        "BPMN business rule task '{node_id}' in process '{process_id}' is missing a DMN decision reference"
    )]
    MissingBusinessRuleDecisionRef {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN node identifier.
        node_id: BpmnErrorId,
    },
    /// Returned when a sequence flow references a missing source or target.
    #[error(
        "BPMN sequence flow '{flow_id}' in process '{process_id}' references unknown {endpoint} '{node_id}'"
    )]
    UnknownSequenceFlowEndpoint {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Sequence flow identifier.
        flow_id: BpmnErrorId,
        /// Endpoint kind.
        endpoint: &'static str,
        /// Missing node identifier.
        node_id: BpmnErrorId,
    },
    /// Returned when the bounded event slice encounters multiple event
    /// definitions on one node.
    #[error(
        "BPMN source '{source_id}' node '{node_id}' in process '{process_id}' uses unsupported multiple event definitions"
    )]
    UnsupportedMultipleEventDefinitions {
        /// Source identifier used for diagnostics.
        source_id: BpmnErrorId,
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN node identifier.
        node_id: BpmnErrorId,
    },
    /// Returned when one boundary event references an unknown attached node.
    #[error(
        "BPMN process '{process_id}' boundary event '{node_id}' references unknown attached node '{attached_to_node_id}'"
    )]
    UnknownBoundaryAttachment {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Boundary-event BPMN node identifier.
        node_id: BpmnErrorId,
        /// Missing attached target node identifier.
        attached_to_node_id: BpmnErrorId,
    },
    /// Returned when one boundary event exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedBoundaryEventConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Boundary-event BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one compensation handler configuration exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' compensation node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedCompensationConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Compensation-boundary or compensation-activity BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one event-based gateway exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' event-based gateway '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedEventBasedGatewayConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Event-based gateway BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one event definition exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' event node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedEventConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Event BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one gateway exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' gateway '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedGatewayConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// Gateway BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one task configuration exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' task node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedTaskConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN task node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one loop configuration exceeds the bounded supported slice.
    #[error(
        "BPMN process '{process_id}' loop node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedLoopConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one call activity references an unknown process id.
    #[error(
        "BPMN process '{process_id}' call activity '{node_id}' references unknown called process '{called_process_id}'"
    )]
    UnknownCalledProcess {
        /// Owning process identifier.
        process_id: BpmnErrorId,
        /// Call-activity BPMN node identifier.
        node_id: BpmnErrorId,
        /// Missing called-process identifier.
        called_process_id: BpmnErrorId,
    },
    /// Returned when one subprocess or call-activity configuration exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' subprocess node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedSubProcessConfiguration {
        /// Owning process identifier.
        process_id: BpmnErrorId,
        /// BPMN subprocess or call-activity node identifier.
        node_id: BpmnErrorId,
        /// Stable unsupported configuration discriminator.
        detail: &'static str,
    },
    /// Returned when one transaction-specific cancel path exceeds the bounded slice.
    #[error(
        "BPMN process '{process_id}' transaction node '{node_id}' uses unsupported configuration '{detail}'"
    )]
    UnsupportedTransactionConfiguration {
        /// Process identifier.
        process_id: BpmnErrorId,
        /// BPMN node identifier.
        node_id: BpmnErrorId,
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
        instance_id: BpmnErrorId,
    },
    /// Returned when one workflow instance has multiple pending host-work
    /// entries but a singleton request surface was used.
    #[error(
        "workflow instance '{instance_id}' has {count} pending host-work entries; a singleton request is ambiguous"
    )]
    AmbiguousPendingHostWork {
        /// Owning workflow instance identifier.
        instance_id: BpmnErrorId,
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
        instance_id: BpmnErrorId,
        /// Missing blocked token identifier.
        token_id: BpmnErrorTokenId,
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
    /// Returned when an explicit pending host-work operation targets a pending
    /// item whose BPMN identity does not match the checkpointed work.
    #[error("{0}")]
    PendingHostWorkIdentityMismatch(Box<BpmnPendingHostWorkIdentityMismatch>),
    /// Returned when a human-task claim request supplies an empty claimant.
    #[error("human task claim requires a non-empty claimant")]
    InvalidHumanTaskClaimant,
    /// Returned when a claim request targets pending host work that is not a
    /// BPMN user or manual task.
    #[error(
        "pending host work token {token_id} node {node_index} has kind '{kind}' and cannot be claimed as a human task"
    )]
    PendingHostWorkNotHumanTask {
        /// Runtime token identifier for the pending host work.
        token_id: BpmnErrorTokenId,
        /// BPMN node index.
        node_index: u32,
        /// Checkpointed host-work kind.
        kind: BpmnErrorKind,
    },
    /// Returned when a different claimant already owns one pending human task.
    #[error("pending host work token {token_id} is already claimed by '{claimed_by}'")]
    PendingHostWorkAlreadyClaimed {
        /// Runtime token identifier for the pending host work.
        token_id: BpmnErrorTokenId,
        /// Existing claimant identifier.
        claimed_by: String,
    },
    /// Returned when a release request targets human work that is not claimed.
    #[error("pending host work token {token_id} is not claimed")]
    PendingHostWorkNotClaimed {
        /// Runtime token identifier for the pending host work.
        token_id: BpmnErrorTokenId,
    },
    /// Returned when a release request is made by a claimant that does not own
    /// the pending human work.
    #[error(
        "pending host work token {token_id} is claimed by '{claimed_by}' and cannot be released by '{requested_by}'"
    )]
    PendingHostWorkClaimReleaseMismatch {
        /// Runtime token identifier for the pending host work.
        token_id: BpmnErrorTokenId,
        /// Existing claimant identifier.
        claimed_by: String,
        /// Claimant supplied by the release request.
        requested_by: String,
    },
    /// Returned when form-backed human-task completion data is not an object.
    #[error(
        "human task completion for process '{process_id}' activity '{activity_id}' must be a JSON object"
    )]
    HumanTaskCompletionDataNotObject {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
    },
    /// Returned when form-backed human-task completion omits a required field.
    #[error(
        "human task completion for process '{process_id}' activity '{activity_id}' is missing required field '{field}'"
    )]
    MissingHumanTaskCompletionField {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
        /// Missing field name.
        field: String,
    },
    /// Returned when form-backed human-task completion submits an undeclared
    /// field.
    #[error(
        "human task completion for process '{process_id}' activity '{activity_id}' contains undeclared field '{field}'"
    )]
    UndeclaredHumanTaskCompletionField {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
        /// Undeclared field name.
        field: String,
    },
    /// Returned when a task input binding references a missing workflow
    /// variable path.
    #[error(
        "task input '{input}' for process '{process_id}' node {node_index} references unresolved source '{source_ref}'"
    )]
    UnresolvedTaskInputSource {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN node index.
        node_index: u32,
        /// Data input name.
        input: String,
        /// Missing source variable path.
        source_ref: String,
    },
    /// Returned when a BPMN `dataObjectReference` points at a missing
    /// process-level `dataObject`.
    #[error(
        "dataObjectReference '{reference_id}' in process '{process_id}' references missing dataObject '{data_object_ref}'"
    )]
    UnknownDataObjectReference {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN `dataObjectReference` identifier.
        reference_id: BpmnErrorId,
        /// Missing BPMN `dataObject` identifier.
        data_object_ref: String,
    },
    /// Returned when host-dispatched task completion has no declared standard
    /// BPMN output mapping.
    #[error(
        "host task completion for process '{process_id}' activity '{activity_id}' requires declared BPMN dataOutputAssociation mappings"
    )]
    MissingTaskOutputMapping {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
    },
    /// Returned when host-dispatched task completion data is not an object.
    #[error(
        "host task completion for process '{process_id}' activity '{activity_id}' must be a JSON object"
    )]
    TaskCompletionDataNotObject {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
    },
    /// Returned when host-dispatched task completion omits a declared output.
    #[error(
        "host task completion for process '{process_id}' activity '{activity_id}' is missing declared output '{field}'"
    )]
    MissingTaskCompletionField {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
        /// Missing output field name.
        field: String,
    },
    /// Returned when host-dispatched task completion submits an undeclared
    /// output.
    #[error(
        "host task completion for process '{process_id}' activity '{activity_id}' contains undeclared output '{field}'"
    )]
    UndeclaredTaskCompletionField {
        /// BPMN process identifier.
        process_id: BpmnErrorId,
        /// BPMN activity identifier.
        activity_id: BpmnErrorId,
        /// Undeclared output field name.
        field: String,
    },
    /// Returned when a checkpoint save attempts to overwrite newer state.
    #[error(
        "checkpoint save for instance '{instance_id}' is stale: attempted sequence {attempted_sequence}, stored sequence {stored_sequence}"
    )]
    StaleCheckpointWrite {
        /// Owning workflow instance identifier.
        instance_id: BpmnErrorId,
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
        instance_id: BpmnErrorId,
    },
    /// Returned when an event-poll operation is attempted without a wait.
    #[error("workflow instance '{instance_id}' does not have any active wait registrations")]
    MissingWaitRegistration {
        /// Owning workflow instance identifier.
        instance_id: BpmnErrorId,
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

impl BpmnEngineError {
    pub(crate) fn pending_host_work_identity_mismatch(
        instance_id: String,
        token_id: u64,
        expected_process_id: String,
        expected_activity_id: String,
        actual_process_id: String,
        actual_activity_id: String,
    ) -> Self {
        Self::PendingHostWorkIdentityMismatch(Box::new(BpmnPendingHostWorkIdentityMismatch {
            instance: instance_id,
            token: token_id,
            expected_process: expected_process_id,
            expected_activity: expected_activity_id,
            actual_process: actual_process_id,
            actual_activity: actual_activity_id,
        }))
    }
}
