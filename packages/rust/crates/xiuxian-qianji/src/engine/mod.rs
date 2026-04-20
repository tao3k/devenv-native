//! Core graph engine based on petgraph.
//! Start in `api`; `compiler` owns manifest compilation.

#[path = "../engine_api.rs"]
mod api;
#[path = "../engine_compiler_api.rs"]
mod compiler_api;

mod compiler;

pub use self::api::{NodeExecutionAffinity, QianjiEdge, QianjiEngine, QianjiNode};
pub use self::compiler_api::QianjiCompiler;
