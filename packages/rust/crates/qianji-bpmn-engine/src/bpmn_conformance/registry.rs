use crate::bpmn_conformance_api::{BpmnConformanceEntry, BpmnConformanceStatus as Status};

const COVERAGE_MATRIX: &str = "full-conformance-coverage.md#coverage-matrix";
const MAINTAIN_EXECUTABLE: &str = "Maintain bounded executable coverage";
const MAINTAIN_METADATA: &str = "Maintain metadata preservation coverage";
const CALLABLE_BINDING: &str = "M4.3 callable binding";
const DATA_OBJECT_EXECUTION: &str = "M4.1 data object execution";
const EVENT_SUBPROCESS: &str = "M4.2 event subprocess v1";
const COLLABORATION_ENVELOPE: &str = "M4.4 collaboration host envelope";
const COMPATIBILITY_SUITE: &str = "M4.5 compatibility suite";
const ADVANCED_CONTROL_FLOW: &str = "M4 advanced control flow";
const STORAGE_POLICY: &str = "Deferred storage policy";

macro_rules! entry {
    (
        $family:literal,
        $status:ident,
        $parser:ident,
        $snapshot:ident,
        $lint:ident,
        $runtime:ident,
        $host_surface:ident,
        $next_milestone:expr
    ) => {
        BpmnConformanceEntry {
            family: $family,
            status: Status::$status,
            parser: Status::$parser,
            snapshot: Status::$snapshot,
            lint: Status::$lint,
            runtime: Status::$runtime,
            host_surface: Status::$host_surface,
            docs_anchor: COVERAGE_MATRIX,
            next_milestone: $next_milestone,
        }
    };
}

pub(crate) const BPMN_CONFORMANCE_REGISTRY: &[BpmnConformanceEntry] = &[
    entry!(
        "Linear process flow",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Host-dispatched tasks",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Human interaction",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Parallel gateway",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Exclusive gateway",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Inclusive gateway",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Event-based gateway",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Complex gateway",
        LintDeferred,
        LintDeferred,
        MetadataOnly,
        LintDeferred,
        LintDeferred,
        MetadataOnly,
        ADVANCED_CONTROL_FLOW
    ),
    entry!(
        "Intermediate catch events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Boundary events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Error and cancel events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Compensation",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Conditional events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Escalation events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Import declarations",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Extension declarations",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Relationship declarations",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Event definition catalogs",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Interfaces/operations",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        CALLABLE_BINDING
    ),
    entry!(
        "Global task catalogs",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        CALLABLE_BINDING
    ),
    entry!(
        "Process callable metadata",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        CALLABLE_BINDING
    ),
    entry!(
        "Callable IO metadata",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        CALLABLE_BINDING
    ),
    entry!(
        "Resource catalogs",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Resource-role metadata",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Flow-element metadata",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Category catalogs",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Terminate events",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Multiple events",
        LintDeferred,
        LintDeferred,
        MetadataOnly,
        LintDeferred,
        LintDeferred,
        MetadataOnly,
        ADVANCED_CONTROL_FLOW
    ),
    entry!(
        "Embedded subprocess",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Call activity",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Transaction",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        MetadataOnly,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Event subprocess",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        EVENT_SUBPROCESS
    ),
    entry!(
        "Standard loop",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Sequential multi-instance",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Parallel multi-instance",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Collaboration and pools",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        COLLABORATION_ENVELOPE
    ),
    entry!(
        "Artifacts",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Lanes",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Item definitions",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        MAINTAIN_METADATA
    ),
    entry!(
        "Data objects",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        DATA_OBJECT_EXECUTION
    ),
    entry!(
        "Data stores",
        LintDeferred,
        MetadataOnly,
        MetadataOnly,
        LintDeferred,
        LintDeferred,
        MetadataOnly,
        STORAGE_POLICY
    ),
    entry!(
        "IO specification",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "Data associations",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
    entry!(
        "BPMN DI",
        MetadataOnly,
        MetadataOnly,
        MetadataOnly,
        Supported,
        MetadataOnly,
        MetadataOnly,
        COMPATIBILITY_SUITE
    ),
    entry!(
        "DMN links",
        BoundedExecutable,
        BoundedExecutable,
        MetadataOnly,
        Supported,
        BoundedExecutable,
        BoundedExecutable,
        MAINTAIN_EXECUTABLE
    ),
];
