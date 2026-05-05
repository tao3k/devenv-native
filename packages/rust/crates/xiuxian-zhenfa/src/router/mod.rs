//! Router traits and method registry for Zhenfa extensions.

#[cfg(feature = "gateway")]
mod extension;
mod registry;
mod traits;

#[cfg(feature = "gateway")]
pub use extension::ZhenfaRouter;
pub use registry::{MethodRegistry, method_handler};
pub use traits::ZhenfaMethodHandler;
