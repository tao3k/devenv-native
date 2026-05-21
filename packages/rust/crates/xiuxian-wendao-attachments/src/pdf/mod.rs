//! PDF-specific attachment acceleration helpers.

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod render;

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod ocr;

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod metrics;

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod profile;

#[cfg(feature = "pdf-source-range")]
mod source_range;

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod structure;

#[cfg(feature = "pdf-source-range")]
#[doc(hidden)]
pub mod text;
