//! Thin host-facing bridge for Julia-owned memory compute surfaces.
//!
//! This module intentionally keeps only the thinnest Wendao-local namespace:
//! the Julia plugin crate owns the typed contracts, host staging, transport,
//! and composed downcalls.
/// Public Wendao boundary.

#[cfg(feature = "julia")]
#[path = "julia/mod.rs"]
pub mod julia;
