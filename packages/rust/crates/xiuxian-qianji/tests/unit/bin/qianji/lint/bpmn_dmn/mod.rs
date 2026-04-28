use super::*;

fn stable_temp_output(output: &str, temp_dir: &TempDir) -> String {
    output.replace(&temp_dir.path().display().to_string(), "$TEMP")
}

fn assert_llm_repair_snapshot_shape(output: &str, expected_fragments: &[&str]) {
    for required_section in [
        "Action:",
        "Fix:",
        "Patch focus:",
        "Examples:",
        "Forbidden forms:",
        "Structured repair:",
        "- strategy:",
        "- contract:",
    ] {
        assert!(
            output.contains(required_section),
            "compact diagnostic should include {required_section}"
        );
    }

    for expected_fragment in expected_fragments {
        assert!(
            output.contains(expected_fragment),
            "compact diagnostic should include {expected_fragment}"
        );
    }
}

mod cases_01;
mod cases_02;
mod cases_03;
mod cases_04;
mod cases_05;
mod cases_06;
mod cases_07;
mod cases_08;
mod cases_09;
mod cases_10;
