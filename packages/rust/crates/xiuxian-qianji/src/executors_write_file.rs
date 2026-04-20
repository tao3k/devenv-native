//! Native file writing mechanism.

#[path = "executors/write_file/mechanism.rs"]
mod mechanism;
#[path = "executors/write_file/pathing.rs"]
mod pathing;
#[path = "executors/write_file/template.rs"]
mod template;

pub use mechanism::WriteFileMechanism;
