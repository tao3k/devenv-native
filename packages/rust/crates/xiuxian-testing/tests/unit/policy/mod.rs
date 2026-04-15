use super::*;
use std::fs;

fn create_temp_crate() -> tempfile::TempDir {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    if let Err(error) = fs::create_dir_all(temp.path().join("src")) {
        panic!("src dir should be created: {error}");
    }
    write_manifest(temp.path(), "");
    temp
}

fn write_manifest(crate_root: &Path, extra: &str) {
    let manifest =
        format!("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{extra}");
    if let Err(error) = fs::write(crate_root.join("Cargo.toml"), manifest) {
        panic!("Cargo.toml should be written: {error}");
    }
}

fn write_fixture_file(crate_root: &Path, relative_path: &str, content: &str) {
    let path = crate_root.join(relative_path);
    let Some(parent) = path.parent() else {
        panic!("fixture path should have parent: {path:?}");
    };
    if let Err(error) = fs::create_dir_all(parent) {
        panic!("fixture directories should be created: {error}");
    }
    if let Err(error) = fs::write(path, content) {
        panic!("fixture file should be written: {error}");
    }
}

mod crate_policy;
mod harness;
mod workspace_config;
