//! Test-only acceleration config parsing helpers.

use crate::llm::acceleration::AccelerationDevice;
use std::path::Path;

/// Parse acceleration device token for tests.
#[must_use]
pub fn parse_acceleration_device_for_tests(raw: Option<&str>) -> Option<AccelerationDevice> {
    crate::llm::acceleration::parse_acceleration_device(raw)
}

/// Resolve acceleration device with explicit/env/config precedence for tests.
#[must_use]
pub fn resolve_acceleration_device_with_for_tests(
    explicit: Option<&str>,
    env_acceleration_device: Option<&str>,
    env_accel_device: Option<&str>,
    config_device: Option<&str>,
) -> AccelerationDevice {
    crate::llm::acceleration::resolve_acceleration_device_with_for_tests(
        explicit,
        env_acceleration_device,
        env_accel_device,
        config_device,
    )
}

/// Load acceleration device from Xiuxian TOML using explicit path roots for tests.
#[must_use]
pub fn load_acceleration_device_with_paths(
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> Option<String> {
    crate::llm::acceleration::load_config_with_paths_for_tests(project_root, config_home)
}
