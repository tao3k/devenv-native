//! Native file writing mechanism.

#[path = "executors_write_file_mechanism.rs"]
mod mechanism;
#[path = "executors/write_file/pathing.rs"]
mod pathing;
#[path = "executors/write_file/template.rs"]
mod template;

pub use mechanism::WriteFileMechanism;
