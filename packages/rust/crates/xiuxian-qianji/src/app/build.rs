use crate::consensus::ConsensusManager;
use crate::engine::QianjiCompiler;
use crate::error::QianjiError;
use crate::scheduler::QianjiScheduler;
use std::sync::Arc;
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(super) fn compile_scheduler(
    manifest_toml: &str,
    index: Arc<LinkGraphIndex>,
    consensus_manager: Option<Arc<ConsensusManager>>,
) -> Result<QianjiScheduler, QianjiError> {
    let compiler = QianjiCompiler::new(index);
    let engine = compiler.compile(manifest_toml)?;
    Ok(QianjiScheduler::with_consensus_manager(
        engine,
        consensus_manager,
    ))
}
