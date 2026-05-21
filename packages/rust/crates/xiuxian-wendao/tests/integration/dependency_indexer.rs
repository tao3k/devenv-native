//! Tests for dependency indexer functionality.

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
fn test_symbol_index_search() {
    let mut index = SymbolIndex::new();

    // Add test symbols
    index.add_symbols(
        "serde",
        &[
            ExternalSymbol {
                name: "Serializer".to_string(),
                kind: SymbolKind::Struct,
                file: std::path::PathBuf::from("lib.rs"),
                line: 10,
                crate_name: "serde".to_string(),
            },
            ExternalSymbol {
                name: "serialize".to_string(),
                kind: SymbolKind::Function,
                file: std::path::PathBuf::from("lib.rs"),
                line: 20,
                crate_name: "serde".to_string(),
            },
        ],
    );

    // Search for "TestStruct"
    let results = index.search("Serializer", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Serializer");
    assert_eq!(results[0].kind, SymbolKind::Struct);
}

#[test]
fn test_dependency_config_load() {
    // Test loading config from actual xiuxian.toml
    let config_path = workspace_root()
        .join("packages/rust/crates/xiuxian-wendao/resources/config/xiuxian.toml");
    let config = DependencyBuildConfig::load(config_path.to_string_lossy().as_ref());

    // Should expose at least one external dependency configuration.
    assert!(!config.manifests.is_empty());

    // Rust dependency configuration should be present.
    let rust_dep = config.manifests.iter().find(|d| d.pkg_type == "rust");
    let Some(rust_dep) = rust_dep else {
        panic!("expected rust dependency manifest entry");
    };
    assert_eq!(rust_dep.registry, Some("cargo".to_string()));
    assert!(
        !rust_dep.manifests.is_empty(),
        "rust dependency config should include at least one manifest glob"
    );
}

#[test]
fn test_external_dependency_new() {
    let dep = ConfigExternalDependency {
        pkg_type: "rust".to_string(),
        registry: Some("cargo".to_string()),
        manifests: vec!["**/Cargo.toml".to_string()],
    };
    // Access inner directly in Rust tests
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
