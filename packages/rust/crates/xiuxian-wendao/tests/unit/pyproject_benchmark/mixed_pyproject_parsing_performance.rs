use std::io::Write as StdWrite;

use super::{generate_pyproject_toml, generate_pyproject_toml_with_extras};
use tempfile::NamedTempFile;
use xiuxian_wendao::parsers::languages::python::pyproject::dependencies::parse_pyproject_dependencies;

#[test]
fn test_mixed_pyproject_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    const FILE_COUNT: usize = 30;

    let start = std::time::Instant::now();
    let mut total_deps = 0;

    for i in 0..FILE_COUNT {
        let dep_count = 25 + (i % 50); // Vary the number of deps

        let content = if i % 3 == 0 {
            generate_pyproject_toml_with_extras(dep_count)
        } else {
            generate_pyproject_toml(dep_count)
        };

        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_pyproject_dependencies(file.path())?;
        total_deps += deps.len();
    }

    let elapsed = start.elapsed();

    // Should complete in under 2 seconds
    let max_duration = super::benchmark_budget(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(3),
    );
    let max_secs = max_duration.as_secs_f64();
    assert!(
        elapsed < max_duration,
        "Mixed pyproject parsing took {:.2}s for {} files, expected < {:.2}s (set {} >= 1.0 to tune)",
        elapsed.as_secs_f64(),
        FILE_COUNT,
        max_secs,
        super::BENCH_SLACK_ENV
    );

    println!(
        "Mixed pyproject parsing: {} files = {:.2}ms ({} total deps)",
        FILE_COUNT,
        elapsed.as_secs_f64() * 1000.0,
        total_deps
    );
    Ok(())
}
