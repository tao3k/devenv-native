//! Integration tests for compiler dispatch route coverage.

use std::path::Path;
use std::sync::Arc;

use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::QianjiCompiler;
use xiuxian_wendao::LinkGraphIndex;

mod annotation_audit;
mod contract_calls;
mod core_dispatch;
mod manifests;
mod wendao_dispatch;

fn build_compiler(index_root: &Path) -> Result<QianjiCompiler, Box<dyn std::error::Error>> {
    let index = Arc::new(LinkGraphIndex::build(index_root)?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    Ok(QianjiCompiler::new(index, orchestrator, registry, None))
}
