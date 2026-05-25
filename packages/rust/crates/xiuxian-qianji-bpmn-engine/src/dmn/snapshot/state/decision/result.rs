//! Result alias for DMN decision snapshot leaves.

use crate::BpmnEngineError;

pub(crate) type Result<T> = std::result::Result<T, BpmnEngineError>;
