//! Security scanning mechanism.

#[path = "executors/security_scan/input.rs"]
mod input;
#[path = "executors_security_scan_mechanism.rs"]
mod mechanism;
#[path = "executors/security_scan/scanner.rs"]
mod scanner;

pub use mechanism::SecurityScanMechanism;
