//! PDF-specific attachment acceleration helpers.

#[cfg(feature = "pdf-inspector")]
#[doc(hidden)]
pub mod audit;

#[cfg(feature = "pdf-render")]
#[doc(hidden)]
pub mod render;

#[cfg(feature = "pdf-render")]
#[doc(hidden)]
pub mod ocr;

#[cfg(feature = "pdf-render")]
mod source_range;

#[cfg(feature = "pdf-inspector")]
#[doc(hidden)]
pub mod structure;
