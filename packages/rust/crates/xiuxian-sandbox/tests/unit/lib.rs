use super::platform::command_in_path;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tempfile::TempDir;

#[test]
fn command_in_path_detects_executable_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let command_path = temp_dir.path().join("nsjail");
    fs::write(&command_path, "#!/bin/sh\nexit 0\n")?;
    #[cfg(unix)]
    fs::set_permissions(&command_path, fs::Permissions::from_mode(0o755))?;

    let path_env = Some(temp_dir.path().as_os_str());
    assert!(command_in_path("nsjail", path_env));

    Ok(())
}

#[cfg(unix)]
#[test]
fn command_in_path_ignores_non_executable_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let command_path = temp_dir.path().join("sandbox-exec");
    fs::write(&command_path, "not executable")?;
    fs::set_permissions(&command_path, fs::Permissions::from_mode(0o644))?;

    let path_env = Some(temp_dir.path().as_os_str());
    assert!(!command_in_path("sandbox-exec", path_env));

    Ok(())
}
