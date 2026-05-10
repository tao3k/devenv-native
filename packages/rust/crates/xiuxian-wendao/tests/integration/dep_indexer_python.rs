//! Integration tests for `xiuxian_wendao::dep_indexer_py` support types.

use std::path::PathBuf;

use xiuxian_wendao::dependency_indexer::{
    ConfigExternalDependency, DependencyBuildConfig, ExternalSymbol, SymbolIndex, SymbolKind,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("failed to resolve workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf()
}

#[test]
fn test_external_dependency_new() {
    let dep = ConfigExternalDependency {
        pkg_type: "rust".to_string(),
        registry: Some("cargo".to_string()),
        manifests: vec!["**/Cargo.toml".to_string()],
    };
    assert_eq!(dep.pkg_type, "rust");
    assert_eq!(dep.registry, Some("cargo".to_string()));
    assert_eq!(dep.manifests, vec!["**/Cargo.toml"]);
}

#[test]
fn test_external_dependency_no_registry() {
    let dep = ConfigExternalDependency {
        pkg_type: "python".to_string(),
        registry: None,
        manifests: vec!["**/pyproject.toml".to_string()],
    };

    assert_eq!(dep.pkg_type, "python");
    assert_eq!(dep.registry, None);
}

#[test]
fn test_symbol_index_search() {
    let mut index = SymbolIndex::new();
    index.add_symbols(
        "test_crate",
        &[
            ExternalSymbol {
                name: "TestStruct".to_string(),
                kind: SymbolKind::Struct,
                file: std::path::PathBuf::from("src/lib.rs"),
                line: 10,
                crate_name: "test_crate".to_string(),
            },
            ExternalSymbol {
                name: "test_function".to_string(),
                kind: SymbolKind::Function,
                file: std::path::PathBuf::from("src/lib.rs"),
                line: 20,
                crate_name: "test_crate".to_string(),
            },
        ],
    );

    let results = index.search("TestStruct", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "TestStruct");
    assert_eq!(results[0].kind, SymbolKind::Struct);
}

#[test]
fn test_dependency_config_load() {
    let config_path = workspace_root()
        .join("packages/rust/crates/xiuxian-wendao/resources/config/xiuxian.toml");
    let config = DependencyBuildConfig::load(config_path.to_string_lossy().as_ref());

    assert!(!config.manifests.is_empty());

    let Some(rust_dep) = config.manifests.iter().find(|d| d.pkg_type == "rust") else {
        panic!("expected rust dependency in config");
    };
    assert_eq!(rust_dep.registry, Some("cargo".to_string()));
    assert!(
        !rust_dep.manifests.is_empty(),
        "rust dependency config should include at least one manifest glob"
    );
}
