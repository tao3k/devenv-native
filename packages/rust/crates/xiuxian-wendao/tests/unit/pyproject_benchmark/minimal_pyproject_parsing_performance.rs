use std::io::Write as StdWrite;

use tempfile::NamedTempFile;
use xiuxian_wendao::parsers::languages::python::pyproject::dependencies::parse_pyproject_dependencies;

#[test]
fn test_minimal_pyproject_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"
[project]
name = "test"
version = "0.1.0"
dependencies = [
    "requests>=2.0",
    "click>=8.0",
    "rich>=13.0",
    "typer>=0.9",
    "pydantic>=2.0",
]
"#;

    let start = std::time::Instant::now();

    // Parse the same content many times
    for _ in 0..100 {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_pyproject_dependencies(file.path())?;
        assert_eq!(deps.len(), 5);
    }

    let elapsed = start.elapsed();

    // Should complete 100 parses in under 300ms
    let max_duration = super::benchmark_budget(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_millis(450),
    );
    let max_duration_ms = max_duration.as_secs_f64() * 1000.0;
    assert!(
        elapsed < max_duration,
        "Minimal pyproject parsing took {:.2}ms for 100 iterations, expected < {:.2}ms (set {} >= 1.0 to tune)",
        elapsed.as_secs_f64() * 1000.0,
        max_duration_ms,
        super::BENCH_SLACK_ENV
    );

    println!(
        "Minimal pyproject parsing: 100 iterations = {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}
