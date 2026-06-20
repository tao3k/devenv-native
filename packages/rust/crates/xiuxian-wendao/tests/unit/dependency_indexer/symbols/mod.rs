use std::io::Write as IoWrite;
use std::path::PathBuf;

use crate::dependency_indexer::{
    ExternalSymbol, SymbolIndex, SymbolKind, extract_dependency_symbols,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn extract_fixture_symbols(
    content: &str,
    language: &str,
) -> Result<Vec<ExternalSymbol>, Box<dyn std::error::Error>> {
    let temp_file = tempfile::NamedTempFile::new()?;
    {
        let mut writer = std::io::BufWriter::new(&temp_file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;
    }
    extract_dependency_symbols(temp_file.path(), language).map_err(Into::into)
}

fn build_symbol_index() -> SymbolIndex {
    let mut index = SymbolIndex::new();
    index.add_symbols(
        "serde",
        &[
            ExternalSymbol {
                name: "Serializer".to_string(),
                kind: SymbolKind::Struct,
                file: PathBuf::from("lib.rs"),
                line: 10,
                crate_name: "serde".to_string(),
            },
            ExternalSymbol {
                name: "serialize".to_string(),
                kind: SymbolKind::Function,
                file: PathBuf::from("lib.rs"),
                line: 20,
                crate_name: "serde".to_string(),
            },
        ],
    );

    index.add_symbols(
        "tokio",
        &[ExternalSymbol {
            name: "spawn".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("lib.rs"),
            line: 5,
            crate_name: "tokio".to_string(),
        }],
    );
    index
}

#[test]
fn test_extract_rust_symbols() -> TestResult {
    let symbols = extract_fixture_symbols(
        r"pub struct MyStruct {
    field: String,
}

pub enum MyEnum {
    Variant,
}

pub fn my_function() {
}
",
        "rust",
    )?;

    assert!(symbols.is_empty());

    Ok(())
}

#[test]
fn test_extract_python_symbols() -> TestResult {
    let symbols = extract_fixture_symbols(
        r"class MyClass:
    pass

def my_function():
    pass
",
        "python",
    )?;

    assert!(symbols.is_empty());

    Ok(())
}

#[test]
fn test_symbol_index_search() {
    let index = build_symbol_index();

    let results = index.search("serialize", 10);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|symbol| symbol.name == "Serializer"));
    assert!(results.iter().any(|symbol| symbol.name == "serialize"));

    let results = index.search("spawn", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "spawn");

    let results = index.search_crate("serde", "serialize", 10);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_serialize_deserialize() {
    let mut index = SymbolIndex::new();

    index.add_symbols(
        "test",
        &[ExternalSymbol {
            name: "MyStruct".to_string(),
            kind: SymbolKind::Struct,
            file: PathBuf::from("lib.rs"),
            line: 10,
            crate_name: "test".to_string(),
        }],
    );

    let data = index.serialize();

    let mut index2 = SymbolIndex::new();
    let _ = index2.deserialize(&data);

    let results = index2.search("MyStruct", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "MyStruct");
}
