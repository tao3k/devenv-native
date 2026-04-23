use crate::contracts::{NodeDefinition, QianjiMechanism};
use std::sync::Arc;

use crate::engine::compiler::io_mechanisms;

pub(in crate::engine::compiler) fn command(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
    let config = io_mechanisms::command_mechanism_config(node_def);
    Arc::new(crate::executors::ShellMechanism {
        cmd: config.cmd,
        allow_fail: config.allow_fail,
        stop_on_empty_stdout: config.stop_on_empty_stdout,
        empty_reason: config.empty_reason,
        output_key: config.output_key,
    })
}

pub(in crate::engine::compiler) fn write_file(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let config = io_mechanisms::write_file_mechanism_config(node_def);
    Arc::new(crate::executors::WriteFileMechanism {
        path: config.path,
        content: config.content,
        output_key: config.output_key,
    })
}

pub(in crate::engine::compiler) fn suspend(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
    let config = io_mechanisms::suspend_mechanism_config(node_def);
    Arc::new(crate::executors::SuspendMechanism {
        reason: config.reason,
        prompt: config.prompt,
        resume_key: config.resume_key,
    })
}
