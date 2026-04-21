//! AST-based Security Scanning Mechanism.

#[path = "executors/security_scan/input.rs"]
mod input;
#[path = "executors_security_scan_mechanism.rs"]
mod mechanism;

pub use mechanism::SecurityScanMechanism;
