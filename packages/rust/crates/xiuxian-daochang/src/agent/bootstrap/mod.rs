//! Agent bootstrap branch wiring for memory, tools, and runtime services.

pub(crate) mod memory;
pub(crate) mod native_tools;
pub(crate) mod qianhuan;
pub(crate) mod service_mount;
pub(crate) mod zhixing;

pub use service_mount::ServiceMountRecord;
