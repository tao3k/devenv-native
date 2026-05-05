use std::io::Write as StdWrite;

use super::generate_pyproject_toml_with_extras;
use tempfile::NamedTempFile;
use xiuxian_wendao::parsers::languages::python::pyproject::dependencies::parse_pyproject_dependencies;

#[test]
fn test_pyproject_extras_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    const DEP_COUNT: usize = 100;

    let start = std::time::Instant::now();

    // Parse multiple pyproject.toml files with extras
    for _ in 0..20 {
        let content = generate_pyproject_toml_with_extras(DEP_COUNT);
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_pyproject_dependencies(file.path())?;
        assert!(!deps.is_empty());
    }

    let elapsed = start.elapsed();

    // Should complete in under 1 second
    let max_duration = super::benchmark_budget(
        std::time::Duration::from_secs(1),
        std::time::Duration::from_millis(1500),
    );
    let max_secs = max_duration.as_secs_f64();
    assert!(
        elapsed < max_duration,
        "pyproject.toml with extras parsing took {:.2}s, expected < {:.2}s (set {} >= 1.0 to tune)",
        elapsed.as_secs_f64(),
        max_secs,
        super::BENCH_SLACK_ENV
    );

    println!(
        "pyproject.toml with extras: 20 files x {} deps = {:.2}ms",
        DEP_COUNT,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}
