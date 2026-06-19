//! Integration tests for compiler dispatch route coverage.

#![cfg(feature = "wendao-integration")]

use std::path::Path;
use std::sync::Arc;

use xiuxian_qianji::QianjiCompiler;
use xiuxian_wendao::LinkGraphIndex;

mod annotation_audit;
mod contract_calls;
mod core_dispatch;
mod manifests;
mod wendao_dispatch;

fn build_compiler(index_root: &Path) -> Result<QianjiCompiler, Box<dyn std::error::Error>> {
    let index = Arc::new(LinkGraphIndex::build(index_root)?);
    Ok(QianjiCompiler::new(index))
}
