//! Canonical api seam for DMN decision snapshot temporary owners.

use super::{attribute_value, required_attribute};
use crate::{
    DmnDecisionSnapshot, DmnFunctionDefinitionLiteralSnapshot,
    DmnFunctionDefinitionParameterSnapshot, DmnFunctionDefinitionSnapshot,
    DmnInvocationBindingSnapshot, DmnInvocationLiteralSnapshot, DmnInvocationParameterSnapshot,
    DmnInvocationSnapshot, DmnRequirementReferenceSnapshot, DmnSourceFile,
};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

mod api;
mod core;
mod function;
mod invocation;
mod requirement;
mod result;
mod text;

pub(super) use result::Result;
use text::non_empty_text;

pub(super) use api::{
    TempDecisionSnapshot, TempFunctionDefinitionLiteralSnapshot,
    TempFunctionDefinitionParameterSnapshot, TempFunctionDefinitionSnapshot,
    TempInvocationBindingSnapshot, TempInvocationLiteralSnapshot, TempInvocationParameterSnapshot,
    TempInvocationSnapshot, TempRequirementReferenceSnapshot,
};
