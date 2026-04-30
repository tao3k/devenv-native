//! Canonical api seam for DMN decision snapshot temporary owners.

use crate::dmn::snapshot::xml::{attribute_value, required_attribute};
use crate::dmn_model_api::{
    DmnDecisionSnapshot, DmnFunctionDefinitionLiteralSnapshot,
    DmnFunctionDefinitionParameterSnapshot, DmnFunctionDefinitionSnapshot,
    DmnInvocationBindingSnapshot, DmnInvocationLiteralSnapshot, DmnInvocationParameterSnapshot,
    DmnInvocationSnapshot, DmnRequirementReferenceSnapshot, DmnSourceFile,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

mod api;
mod core;
mod function;
mod invocation;
mod requirement;
mod text;

use text::non_empty_text;

pub(super) use api::{
    TempDecisionSnapshot, TempFunctionDefinitionLiteralSnapshot,
    TempFunctionDefinitionParameterSnapshot, TempFunctionDefinitionSnapshot,
    TempInvocationBindingSnapshot, TempInvocationLiteralSnapshot, TempInvocationParameterSnapshot,
    TempInvocationSnapshot, TempRequirementReferenceSnapshot,
};
