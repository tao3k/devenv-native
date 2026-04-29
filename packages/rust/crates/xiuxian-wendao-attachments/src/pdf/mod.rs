//! PDF-specific attachment acceleration helpers.

#[cfg(feature = "pdf-inspector")]
#[doc(hidden)]
pub mod audit;

#[cfg(feature = "pdf-render")]
#[doc(hidden)]
pub mod render;
