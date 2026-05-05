use std::io::Write as StdWrite;

use tempfile::NamedTempFile;
use xiuxian_wendao::parsers::languages::python::pyproject::dependencies::parse_pyproject_dependencies;

#[test]
fn test_regex_fallback_parsing_performance() -> Result<(), Box<dyn std::error::Error>> {
    // This tests the regex fallback path (when TOML parsing fails)
    let content =
        "package1==1.0.0\npackage2>=2.0.0\npackage3~=4.0.0\nanother_package[extra]==5.0.0\n";

    let start = std::time::Instant::now();

    for _ in 0..100 {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;

        let deps = parse_pyproject_dependencies(file.path())?;
        assert_eq!(deps.len(), 4);
    }

    let elapsed = start.elapsed();

    // Should complete 100 parses in under 200ms
    let max_duration = super::benchmark_budget(
        std::time::Duration::from_millis(200),
        std::time::Duration::from_millis(300),
    );
    let max_duration_ms = max_duration.as_secs_f64() * 1000.0;
    assert!(
        elapsed < max_duration,
        "Regex fallback parsing took {:.2}ms for 100 iterations, expected < {:.2}ms (set {} >= 1.0 to tune)",
        elapsed.as_secs_f64() * 1000.0,
        max_duration_ms,
        super::BENCH_SLACK_ENV
    );

    println!(
        "Regex fallback parsing: 100 iterations = {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}
