use std::path::{Path, PathBuf};
use std::process::Command;

const FORBIDDEN_CONTRACT_DEPENDENCIES: &[&str] = &[
    "axum",
    "tonic",
    "arrow-flight",
    "duckdb",
    "datafusion",
    "notify",
    "xiuxian-db-store",
];

#[test]
fn contracts_feature_keeps_runtime_dependencies_out_of_normal_tree() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root(&manifest_dir);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = match Command::new(cargo)
        .current_dir(workspace_root)
        .args([
            "tree",
            "-p",
            "xiuxian-wendao-studio",
            "--no-default-features",
            "--features",
            "contracts",
            "-e",
            "normal",
        ])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            panic!("failed to run cargo tree for xiuxian-wendao-studio contracts feature: {error}")
        }
    };

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

fn workspace_root(manifest_dir: &Path) -> &Path {
    match manifest_dir.ancestors().nth(4) {
        Some(path) => path,
        None => panic!("xiuxian-wendao-studio should live under packages/rust/crates"),
    }
}
