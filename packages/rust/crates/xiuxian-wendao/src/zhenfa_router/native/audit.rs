//! Context Completeness Score (CCS) audit module for wendao.search.
//!
//! Implements Synapse-Audit (2025) principles for context quality gating.
//! This module is intentionally self-contained to avoid circular dependencies
//! with xiuxian-qianhuan.
//!
//! ## Module Structure
//!
//! - `ccs` - Core CCS calculation and drift scoring
//! - `verdict` - Audit verdict and result types
//! - `compensation` - Compensation request for secondary search
//! - `fuzzy_suggest` - Fuzzy pattern suggestion for code observations (Blueprint v2.9)
//! - `audit_bridge` - Batch fix generation with surgical precision (Blueprint v3.0)
//! - `fix` - CLI fix tool with atomic write-back (Blueprint v3.1)
//!
//! ## Usage
//!
//! ```ignore
//! use crate::zhenfa_router::native::audit::{audit_search_payload, AuditResult};
//!
//! let result = audit_search_payload(&evidence, &anchors);
//! if !result.passed {
//!     // Apply compensation
//! }
//! ```

#[path = "audit/audit_bridge/mod.rs"]
mod audit_bridge;
#[path = "audit/ccs.rs"]
mod ccs;
#[path = "audit/compensation.rs"]
mod compensation;
#[path = "audit/fix/mod.rs"]
pub mod fix;
#[path = "audit/fuzzy_suggest/mod.rs"]
mod fuzzy_suggest;
#[path = "audit/verdict.rs"]
mod verdict;

#[allow(unused_imports)] // Will be used by qianji integration
pub use audit_bridge::{
    BatchFix, ByteRange, FixResult, generate_batch_fixes, generate_surgical_fixes,
};
pub use ccs::{audit_search_payload, evaluate_alignment};
pub use compensation::CompensationRequest;
pub use fuzzy_suggest::{
    FuzzySuggestion, SourceFile, cache_stats, clear_candidate_cache, format_suggestion,
    resolve_source_files, suggest_pattern_fix, suggest_pattern_fix_with_threshold,
};
pub use verdict::AuditResult;
