use crate::dependency_indexer::{ConfigExternalDependency, SymbolIndex};
use std::path::PathBuf;

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

#[test]
fn test_symbol_index_search() {
    let mut index = SymbolIndex::new();
    index.add_symbols(
        "test_crate",
        &[
            crate::dependency_indexer::ExternalSymbol {
                name: "TestStruct".to_string(),
                kind: crate::dependency_indexer::SymbolKind::Struct,
                file: std::path::PathBuf::from("src/lib.rs"),
                line: 10,
                crate_name: "test_crate".to_string(),
            },
            crate::dependency_indexer::ExternalSymbol {
                name: "test_function".to_string(),
                kind: crate::dependency_indexer::SymbolKind::Function,
                file: std::path::PathBuf::from("src/lib.rs"),
                line: 20,
                crate_name: "test_crate".to_string(),
            },
        ],
    );

    // Search for "TestStruct"
    let results = index.search("TestStruct", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "TestStruct");
    assert_eq!(
        results[0].kind,
        crate::dependency_indexer::SymbolKind::Struct
    );
}

#[test]
fn test_dependency_config_load() {
    use crate::dependency_indexer::DependencyBuildConfig as ConfigType;

    // Test loading config from actual xiuxian.toml
    let config_path =
        workspace_root().join("packages/rust/crates/xiuxian-wendao/resources/config/xiuxian.toml");
    let config = ConfigType::load(config_path.to_string_lossy().as_ref());

    // Should have at least one external dependency configuration.
    assert!(!config.manifests.is_empty());

    // Find rust dependency
    let Some(rust_dep) = config.manifests.iter().find(|d| d.pkg_type == "rust") else {
        panic!("expected rust dependency in config");
    };
    assert_eq!(rust_dep.registry, Some("cargo".to_string()));
}
