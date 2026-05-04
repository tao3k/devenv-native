use std::path::{Path, PathBuf};
use std::process::Command;

const FORBIDDEN_NATIVE_DEPENDENCIES: &[&str] =
    &["axum", "reqwest", "quick-xml", "tempfile", "pulldown-cmark"];

#[test]
fn no_default_features_keep_http_and_validation_dependencies_out_of_normal_tree() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root(&manifest_dir);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .current_dir(workspace_root)
        .args([
            "tree",
            "-p",
            "xiuxian-zhenfa",
            "--no-default-features",
            "-e",
            "normal",
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to run cargo tree for xiuxian-zhenfa no-default feature set: {error}")
        });

    assert!(
        output.status.success(),
        "cargo tree failed for xiuxian-zhenfa no-default feature set:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tree = String::from_utf8_lossy(&output.stdout);
    for dependency in FORBIDDEN_NATIVE_DEPENDENCIES {
        let marker = format!("{dependency} v");
        assert!(
            !tree.lines().any(|line| line.contains(marker.as_str())),
            "xiuxian-zhenfa no-default feature set must not pull `{dependency}` into the normal dependency tree:\n{tree}"
        );
    }
}

fn workspace_root(manifest_dir: &Path) -> &Path {
    match manifest_dir.ancestors().nth(4) {
        Some(path) => path,
        None => panic!("xiuxian-zhenfa should live under packages/rust/crates"),
    }
}
