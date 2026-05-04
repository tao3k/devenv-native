use std::path::Path;

use super::support::{cargo_tree, workspace_root};

const FORBIDDEN_CONTRACT_DEPENDENCIES: &[&str] = &[
    "axum",
    "tonic",
    "arrow-flight",
    "duckdb",
    "datafusion",
    "notify",
    "xiuxian-db-store",
    "xiuxian-wendao-core",
    "chrono",
    "inventory",
    "toml",
];
const FORBIDDEN_LOCAL_RUNTIME_ZHENFA_FEATURES: &[&str] =
    &["gateway", "client", "contract-validation", "xml-transform"];

#[test]
fn contracts_feature_keeps_runtime_dependencies_out_of_normal_tree() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = cargo_tree(
        workspace_root(manifest_dir),
        [
            "-p",
            "xiuxian-wendao-studio",
            "--no-default-features",
            "--features",
            "contracts",
            "-e",
            "normal",
        ],
    );

    assert!(
        output.status.success(),
        "cargo tree failed for contracts feature:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tree = String::from_utf8_lossy(&output.stdout);
    for dependency in FORBIDDEN_CONTRACT_DEPENDENCIES {
        let marker = format!("{dependency} v");
        assert!(
            !tree.lines().any(|line| line.contains(marker.as_str())),
            "contracts feature must not pull `{dependency}` into the normal dependency tree:\n{tree}"
        );
    }
}

#[test]
fn local_runtime_keeps_zhenfa_gateway_features_out_of_feature_tree() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = cargo_tree(
        workspace_root(manifest_dir),
        [
            "-p",
            "xiuxian-wendao-studio",
            "--no-default-features",
            "--features",
            "local-runtime",
            "-e",
            "features",
        ],
    );

    assert!(
        output.status.success(),
        "cargo tree failed for local-runtime feature:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tree = String::from_utf8_lossy(&output.stdout);
    for feature in FORBIDDEN_LOCAL_RUNTIME_ZHENFA_FEATURES {
        let marker = format!("xiuxian-zhenfa feature \"{feature}\"");
        assert!(
            !tree.lines().any(|line| line.contains(marker.as_str())),
            "local-runtime feature must not enable xiuxian-zhenfa `{feature}`:\n{tree}"
        );
    }
}
