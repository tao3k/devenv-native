use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static WENDAO_CONFIG_HOME_OVERRIDE: OnceLock<RwLock<Option<String>>> = OnceLock::new();
static WENDAO_CONFIG_OVERRIDE: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn config_home_override_store() -> &'static RwLock<Option<String>> {
    WENDAO_CONFIG_HOME_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn wendao_config_override_store() -> &'static RwLock<Option<String>> {
    WENDAO_CONFIG_OVERRIDE.get_or_init(|| RwLock::new(None))
}

pub(crate) fn set_wendao_config_home_override(path: &str) {
    let mut guard = match config_home_override_store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(path.trim().to_string());
}

pub(crate) fn clear_wendao_config_home_override() {
    let mut guard = match config_home_override_store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = None;
}

pub(crate) fn set_wendao_config_override(path: &str) {
    let mut guard = match wendao_config_override_store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(path.trim().to_string());
}

pub(crate) fn clear_wendao_config_override() {
    let mut guard = match wendao_config_override_store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = None;
}

#[must_use]
pub(crate) fn wendao_config_file_override() -> Option<PathBuf> {
    let guard = match wendao_config_override_store().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clone().map(PathBuf::from)
}

#[must_use]
pub(crate) fn wendao_config_home_override() -> Option<PathBuf> {
    let guard = match config_home_override_store().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clone().map(PathBuf::from)
}
