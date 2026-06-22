use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn read_crate_source(relative_path: &str) -> String {
    let path = crate_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read crate source {}: {error}", path.display()))
}

pub(super) fn assert_live_bridge_source_is_gateway_config_free(relative_path: &str) {
    let source = read_crate_source(relative_path);
    for forbidden in [
        "wendao.toml",
        "ROOT_WENDAO_CONFIG",
        "ROOT_WENDAO_CONFIG_PATH",
        "resolve_wendao_config",
        "Gateway route registration",
    ] {
        assert!(
            !source.contains(forbidden),
            "{relative_path} must not contain live Gateway config or route-owner marker {forbidden}"
        );
    }
}

pub(super) fn display_relative(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .display()
        .to_string()
}
