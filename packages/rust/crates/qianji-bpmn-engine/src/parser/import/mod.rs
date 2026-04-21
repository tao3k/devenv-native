//! BPMN source ingestion and XML extraction.

mod attributes;
mod capture;
mod model;
mod nested;
mod process;
mod reader;

pub(crate) use reader::{
    NestedShellKind, RawAssociation, RawEventSpec, RawNode, RawPackageDocument,
    RawParallelMultiInstanceSpec, RawProcess, RawProcessScope, RawRepeatSpec, RawSequenceFlow,
    RawSequentialMultiInstanceSpec, RawSubProcessKind, import_bpmn_source,
};
