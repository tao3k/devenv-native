use std::sync::Arc;

use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::engine::compiler::io_mechanisms;
use crate::error::QianjiError;

pub(in crate::engine::compiler) fn http_call(
    node_def: &NodeDefinition,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let config = io_mechanisms::http_call_mechanism_config(node_def)?;
    Ok(Arc::new(crate::executors::HttpCallMechanism {
        contract: config.contract,
        method: config.method,
        path: config.path,
        base_url: config.base_url,
        query: config.query,
        output_key: config.output_key,
    }))
}

pub(in crate::engine::compiler) fn cli_call(
    node_def: &NodeDefinition,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let config = io_mechanisms::cli_call_mechanism_config(node_def)?;
    Ok(Arc::new(crate::executors::CliCallMechanism {
        contract: config.contract,
        argv: config.argv,
        output_key: config.output_key,
    }))
}
