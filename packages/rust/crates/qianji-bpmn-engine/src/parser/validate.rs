//! BPMN validation root seam.
//!
//! Start with `package` for package-level orchestration.

mod boundary;
mod error_paths;
mod escalation_paths;
mod package;
mod recursion;

pub(crate) use self::package::{resolve_structured_inclusive_join, validate_raw_package};
