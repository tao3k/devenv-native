//! BPMN source ingestion and XML extraction.

mod attributes;
mod capture;
mod human_task_io;
mod lane;
mod model;
mod nested;
mod process;
mod reader;
mod task_io;

pub(crate) use lane::attach_lane_memberships;
pub(crate) use model::{
    RawHumanTaskAssignmentSpec, RawHumanTaskChoiceSpec, RawHumanTaskFormSpec,
    RawHumanTaskFreeTextSpec, RawHumanTaskNativeIoSpec, RawHumanTaskResourceRoleKind,
    RawHumanTaskResourceRoleSpec, RawLaneMembershipSpec, RawTaskInputBinding, RawTaskInputSource,
    RawTaskIoSpec, RawTaskOutputBinding,
};
pub(crate) use reader::{
    NestedShellKind, RawAssociation, RawEventSpec, RawNode, RawPackageDocument,
    RawParallelMultiInstanceSpec, RawProcess, RawProcessScope, RawRepeatSpec, RawScriptTaskSpec,
    RawSequenceFlow, RawSequentialMultiInstanceSpec, RawSubProcessKind, import_bpmn_source,
};
