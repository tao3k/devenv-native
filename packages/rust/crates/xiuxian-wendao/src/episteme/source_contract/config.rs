//! Compatibility facade for Episteme-owned runtime defaults.

use std::path::Path;

use super::EpistemeError;

pub use xiuxian_wendao_episteme::EpistemeRuntimeConfig;

/// Load optional Episteme runtime defaults from `<episteme-root>/episteme.toml`.
///
/// # Errors
///
/// Returns an error when `episteme.toml` exists but cannot be read or parsed.
pub fn load_episteme_runtime_config(
    episteme_root: impl AsRef<Path>,
) -> Result<Option<EpistemeRuntimeConfig>, EpistemeError> {
    xiuxian_wendao_episteme::load_episteme_runtime_config(episteme_root).map_err(Into::into)
}
