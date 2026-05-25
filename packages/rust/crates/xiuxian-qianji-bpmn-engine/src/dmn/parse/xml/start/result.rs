//! Result alias for DMN XML start-element parsing leaves.

use crate::BpmnEngineError;

pub(crate) type Result<T> = std::result::Result<T, BpmnEngineError>;
