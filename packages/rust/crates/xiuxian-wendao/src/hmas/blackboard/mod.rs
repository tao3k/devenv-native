//! `hmas::blackboard` owns Wendao hmas blackboard behavior.

mod ast;
mod extract;
mod report;
mod validate;

pub use report::{HmasValidationIssue, HmasValidationReport};
pub use validate::{validate_blackboard_file, validate_blackboard_markdown};
