use std::io::Write;
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;
use xiuxian_wendao::dependency_indexer::extract_symbols;

use super::generate_rust_test_file;

#[test]
fn test_rust_symbol_extraction_performance() -> Result<(), Box<dyn std::error::Error>> {
    const FILE_COUNT: usize = 50;
    const LINES_PER_FILE: usize = 500;

    let start = Instant::now();

    // Create and process multiple test files
    let mut temp_files = Vec::new();
    let mut all_symbols = Vec::new();

    for _ in 0..FILE_COUNT {
        let content = generate_rust_test_file(LINES_PER_FILE);

        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        let path = file.path().to_path_buf();

        let symbols = extract_symbols(&path, "rust")?;
        all_symbols.extend(symbols);
        temp_files.push(file);
    }

    let elapsed = start.elapsed();

    // Verify we extracted a reasonable number of symbols
    assert!(!all_symbols.is_empty(), "Should extract symbols");

    // Performance assertion: should process 50 files with 500 lines each in under 2 seconds
    // This is generous to account for slower CI environments
    let max_duration = Duration::from_secs(2);
    assert!(
        elapsed < max_duration,
        "Rust symbol extraction took {:.2}s, expected < 2s for {} files x {} lines",
        elapsed.as_secs_f64(),
        FILE_COUNT,
        LINES_PER_FILE
    );

    println!(
        "Rust symbol extraction: {} files x {} lines = {:.2}ms ({} symbols extracted)",
        FILE_COUNT,
        LINES_PER_FILE,
        elapsed.as_secs_f64() * 1000.0,
        all_symbols.len()
    );
    Ok(())
}
