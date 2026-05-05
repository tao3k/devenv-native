//! Benchmark tests for pyproject.toml parsing performance.
//!
//! These tests measure the performance of parsing pyproject.toml files
//! for Python dependency extraction.

use std::fmt::Write as FmtWrite;

const BENCH_SLACK_ENV: &str = "OMNI_WENDAO_BENCH_SLACK_FACTOR";
const DEFAULT_BENCH_SLACK_FACTOR: f64 = 2.0;

fn benchmark_slack_factor() -> f64 {
    std::env::var(BENCH_SLACK_ENV)
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|factor| factor.is_finite() && *factor >= 1.0)
        .unwrap_or(DEFAULT_BENCH_SLACK_FACTOR)
}

fn benchmark_runtime_multiplier() -> f64 {
    if std::env::var_os("NEXTEST_RUN_ID").is_some() {
        6.0
    } else {
        1.0
    }
}

fn benchmark_budget(local: std::time::Duration, ci: std::time::Duration) -> std::time::Duration {
    let baseline = if std::env::var_os("CI").is_some() {
        ci
    } else {
        local
    };
    baseline.mul_f64(benchmark_slack_factor() * benchmark_runtime_multiplier())
}

fn append_format(content: &mut String, args: std::fmt::Arguments<'_>) {
    if content.write_fmt(args).is_err() {
        unreachable!("formatting into String should not fail");
    }
}

/// Generate a pyproject.toml with many dependencies.
fn generate_pyproject_toml(dep_count: usize) -> String {
    let mut content = String::from(
        "[project]\nname = \"test-project\"\nversion = \"0.1.0\"\ndescription = \"A test project\"\nrequires-python = \">=3.10\"\ndependencies = [\n",
    );

    // Add dependencies with various version specifiers
    for i in 0..dep_count {
        append_format(
            &mut content,
            format_args!(
                "    \"package{i}=={}.{}.{}\",\n",
                i / 100,
                (i / 10) % 10,
                i % 10
            ),
        );
    }

    content.push_str("]\n\n[project.optional-dependencies]\ndev = [\n");
    for i in 0..(dep_count / 3) {
        append_format(
            &mut content,
            format_args!("    \"dev_package{i}>=1.0.0\",\n"),
        );
    }
    content.push_str("]\n");

    content
}

/// Generate a complex pyproject.toml with extras.
fn generate_pyproject_toml_with_extras(dep_count: usize) -> String {
    let mut content =
        String::from("[project]\nname = \"test-project\"\nversion = \"0.1.0\"\ndependencies = [\n");

    // Add dependencies with extras (e.g., package[extra]==version)
    for i in 0..dep_count {
        let extra = if i % 5 == 0 {
            "ssl"
        } else if i % 5 == 1 {
            "cli"
        } else if i % 5 == 2 {
            "dev"
        } else {
            "full"
        };
        append_format(
            &mut content,
            format_args!(
                "    \"package{i}[{extra}]=={}.{}.{}\",\n",
                i / 100,
                (i / 10) % 10,
                i % 10
            ),
        );
    }

    content.push_str("]\n");
    content
}

mod minimal_pyproject_parsing_performance;
mod mixed_pyproject_parsing_performance;
mod pyproject_extras_parsing_performance;
/// Benchmark test for parsing pyproject.toml with many dependencies.
mod pyproject_parsing_performance;
mod regex_fallback_parsing_performance;
