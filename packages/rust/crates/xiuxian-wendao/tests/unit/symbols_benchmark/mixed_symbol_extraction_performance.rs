use std::io::Write;
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;
use xiuxian_wendao::dependency_indexer::extract_dependency_symbols;

use super::{generate_python_test_file, generate_rust_test_file};

#[test]
fn test_mixed_symbol_extraction_performance() -> Result<(), Box<dyn std::error::Error>> {
    const TOTAL_FILES: usize = 100; // 50 Rust + 50 Python

    let start = Instant::now();

    let mut all_symbols = Vec::new();

    // Process Rust files
    for _ in 0..(TOTAL_FILES / 2) {
        let content = generate_rust_test_file(250);
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        let path = file.path().to_path_buf();
        let symbols = extract_dependency_symbols(&path, "rust")?;
        all_symbols.extend(symbols);
    }

    // Process Python files
    for _ in 0..(TOTAL_FILES / 2) {
        let content = generate_python_test_file(250);
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        let path = file.path().to_path_buf();
        let symbols = extract_dependency_symbols(&path, "python")?;
        all_symbols.extend(symbols);
    }

    let elapsed = start.elapsed();

    // Performance assertion
    let max_duration = Duration::from_secs(3);
    assert!(
        elapsed < max_duration,
        "Mixed symbol extraction took {:.2}s, expected < 3s",
        elapsed.as_secs_f64()
    );

    println!(
        "Mixed symbol extraction: {} files = {:.2}ms ({} symbols)",
        TOTAL_FILES,
        elapsed.as_secs_f64() * 1000.0,
        all_symbols.len()
    );
    Ok(())
}
