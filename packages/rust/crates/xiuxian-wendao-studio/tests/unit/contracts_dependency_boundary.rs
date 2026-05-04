use std::fs;
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
    "xiuxian-wendao-core",
    "chrono",
    "inventory",
    "toml",
];
const FORBIDDEN_LOCAL_RUNTIME_ZHENFA_FEATURES: &[&str] =
    &["gateway", "client", "contract-validation", "xml-transform"];
const DOMAIN_CONTRACT_IMPORT_HEAD: &str = "xiuxian_wendao::search";
const DOMAIN_CONTRACT_IMPORT_TAIL: &str = "::contracts";

#[test]
fn contracts_feature_keeps_runtime_dependencies_out_of_normal_tree() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root(&manifest_dir);
    let output = cargo_tree(
        workspace_root,
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
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root(&manifest_dir);
    let output = cargo_tree(
        workspace_root,
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

#[test]
fn studio_code_uses_studio_contract_import_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let allowed_reexport = manifest_dir.join("src/contracts/types.rs");
    let needle = format!("{DOMAIN_CONTRACT_IMPORT_HEAD}{DOMAIN_CONTRACT_IMPORT_TAIL}");
    let mut offenders = Vec::new();

    for relative_root in ["src", "tests"] {
        collect_domain_contract_imports(
            manifest_dir.join(relative_root).as_path(),
            allowed_reexport.as_path(),
            needle.as_str(),
            &mut offenders,
        );
    }

    assert!(
        offenders.is_empty(),
        "Studio code should import Studio API contracts through crate::contracts or xiuxian_wendao_studio::contracts; only src/contracts/types.rs may re-export the domain transition path:\n{}",
        offenders.join("\n")
    );
}

fn cargo_tree<const N: usize>(workspace_root: &Path, args: [&str; N]) -> std::process::Output {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
        .current_dir(workspace_root)
        .arg("tree")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run cargo tree: {error}"))
}

fn collect_domain_contract_imports(
    root: &Path,
    allowed_reexport: &Path,
    needle: &str,
    offenders: &mut Vec<String>,
) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_domain_contract_imports(path.as_path(), allowed_reexport, needle, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if path == allowed_reexport {
            continue;
        }
        let source = fs::read_to_string(path.as_path())
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if source.contains(needle) {
            offenders.push(path.display().to_string());
        }
    }
}

fn workspace_root(manifest_dir: &Path) -> &Path {
    match manifest_dir.ancestors().nth(4) {
        Some(path) => path,
        None => panic!("xiuxian-wendao-studio should live under packages/rust/crates"),
    }
}
