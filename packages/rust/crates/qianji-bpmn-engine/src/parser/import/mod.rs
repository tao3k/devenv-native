//! BPMN source ingestion and XML extraction.

mod api;
mod attributes;
mod capture;
mod human_task_io;
mod lane;
mod model;
mod nested;
mod process;
mod reader;
mod task_io;

pub(crate) use api::{
    NestedShellKind, RawAssociation, RawEventSpec, RawHumanTaskAssignmentSpec,
    RawHumanTaskChoiceSpec, RawHumanTaskFormSpec, RawHumanTaskFreeTextSpec,
    RawHumanTaskNativeIoSpec, RawHumanTaskResourceRoleKind, RawHumanTaskResourceRoleSpec,
    RawLaneMembershipSpec, RawNode, RawPackageDocument, RawParallelMultiInstanceSpec, RawProcess,
    RawProcessScope, RawRepeatSpec, RawScriptTaskSpec, RawSequenceFlow,
    RawSequentialMultiInstanceSpec, RawSubProcessKind, RawTaskInputBinding, RawTaskInputSource,
    RawTaskIoSpec, RawTaskOutputBinding, attach_lane_memberships, import_bpmn_source,
};
