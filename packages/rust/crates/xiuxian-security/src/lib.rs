//! xiuxian-security - Security Scanner & Sandbox for Omni `DevEnv`
//!
//! ## Modules
//!
//! - `security`: Secret scanning and permission gatekeeper
//! - `sandbox`: Isolated execution environment for harvested skills
//!
//! ## Features
//!
//! - O(n) linear-time regex matching via `RegexSet`
//! - Pre-compiled patterns at startup (Lazy static)
//! - Zero-copy scanning for large files
//! - Fail-fast on first detected secret
//! - Docker/NsJail sandboxing for safe test execution
//!
//! Patterns follow ODF-REP Security Standards.

mod permissions;
mod sandbox;
mod scanner;

pub use permissions::PermissionGatekeeper;
pub use sandbox::{SandboxConfig, SandboxError, SandboxMode, SandboxResult, SandboxRunner};
pub use scanner::{SecretScanner, SecurityViolation};
