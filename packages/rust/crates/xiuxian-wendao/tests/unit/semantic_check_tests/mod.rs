//! Unit tests for `semantic_check` module (Blueprint v2.2).

pub(super) use super::*;
pub(super) use crate::link_graph::{PageIndexMeta, PageIndexNode};
pub(super) use crate::parsers::markdown::CodeObservation;
pub(super) use std::sync::Arc;

mod contract_validation;
mod extraction;
mod health_score;
mod helper_functions;
mod observation_checks;
mod support;
