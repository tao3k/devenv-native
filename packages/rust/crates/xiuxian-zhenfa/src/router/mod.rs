//! Router traits and method registry for Zhenfa extensions.

mod extension;
mod registry;
mod traits;

pub use extension::ZhenfaRouter;
pub use registry::{MethodRegistry, method_handler};
pub use traits::ZhenfaMethodHandler;
