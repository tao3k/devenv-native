//! Tests for xiuxian-macros.

mod test_patterns {
    use xiuxian_macros::patterns;

    patterns![
        (TEST_PATTERN_1, "pattern one"),
        (TEST_PATTERN_2, "pattern two"),
    ];

    #[test]
    fn test_patterns_generated() {
        assert_eq!(TEST_PATTERN_1, "pattern one");
        assert_eq!(TEST_PATTERN_2, "pattern two");
    }
}

mod test_topics {
    use xiuxian_macros::topics;

    topics![(TOPIC_ONE, "topic/one"), (TOPIC_TWO, "topic/two"),];

    #[test]
    fn test_topics_generated() {
        assert_eq!(TOPIC_ONE, "topic/one");
        assert_eq!(TOPIC_TWO, "topic/two");
    }
}

mod test_py_from {
    use xiuxian_macros::py_from;

    struct Inner {
        value: i32,
    }

    struct PyWrapper {
        inner: Inner,
    }

    py_from!(PyWrapper, Inner);

    #[test]
    fn test_py_from_generated() {
        let inner = Inner { value: 42 };
        let wrapper = PyWrapper::from(inner);
        assert_eq!(wrapper.inner.value, 42);
    }
}

mod test_temp_dir {
    use std::fs;
    use xiuxian_macros::temp_dir;

    #[test]
    fn test_temp_dir_creates_directory() {
        let temp_path = temp_dir!();
        assert!(temp_path.exists());
        assert!(temp_path.is_dir());

        fs::remove_dir_all(&temp_path)
            .unwrap_or_else(|error| panic!("remove temp dir {}: {error}", temp_path.display()));
    }

    #[test]
    fn test_temp_dir_is_unique() {
        let temp_path1 = temp_dir!();
        let temp_path2 = temp_dir!();

        assert_ne!(temp_path1, temp_path2);

        if let Err(error) = fs::remove_dir_all(&temp_path1) {
            panic!("remove temp dir {}: {error}", temp_path1.display());
        }
        if let Err(error) = fs::remove_dir_all(&temp_path2) {
            panic!("remove temp dir {}: {error}", temp_path2.display());
        }
    }
}

mod test_assert_timing {
    use xiuxian_macros::assert_timing;

    #[test]
    fn test_assert_timing_passes_fast_operation() {
        let elapsed = assert_timing!(100.0, {
            let x = 1 + 1;
            assert_eq!(x, 2);
        });
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_assert_timing_returns_elapsed() {
        let elapsed = assert_timing!(1000.0, {
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        assert!(elapsed.as_millis() >= 1);
    }
}

mod test_bench_case {
    use xiuxian_macros::bench_case;

    #[test]
    fn test_bench_case_measures_time() {
        let elapsed = bench_case!({
            let sum: i32 = (0..100).sum();
            assert_eq!(sum, 4950);
        });
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn test_bench_case_simple() {
        let _elapsed = bench_case!(1 + 1);
    }
}

mod test_project_config_paths {
    use std::path::PathBuf;
    use std::process::Command;
    use xiuxian_macros::project_config_paths;

    fn run_child_test(test_name: &str, envs: &[(&str, &str)]) {
        let current_exe = std::env::current_exe()
            .unwrap_or_else(|error| panic!("resolve current xiuxian-macros test binary: {error}"));
        let output = Command::new(current_exe)
            .arg("--exact")
            .arg(test_name)
            .arg("--ignored")
            .arg("--nocapture")
            .envs(envs.iter().copied())
            .output()
            .unwrap_or_else(|error| panic!("run child xiuxian-macros test {test_name}: {error}"));

        assert!(
            output.status.success(),
            "child test {test_name} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn test_project_config_paths_generates_layered_candidates() {
        run_child_test(
            "test_project_config_paths::child_project_config_paths_generates_layered_candidates",
            &[
                ("PRJ_ROOT", "/tmp/omni-macro-prj"),
                ("PRJ_CONFIG_HOME", "/tmp/omni-macro-conf"),
                ("QIANJI_CONFIG_PATH", "/tmp/custom/qianji.toml"),
            ],
        );
    }

    #[test]
    #[ignore = "spawned by test_project_config_paths_generates_layered_candidates with isolated env"]
    fn child_project_config_paths_generates_layered_candidates() {
        let paths = project_config_paths!("qianji.toml", "QIANJI_CONFIG_PATH");
        assert_eq!(paths.len(), 3);
        assert_eq!(
            paths[0],
            PathBuf::from("/tmp/omni-macro-prj/packages/conf/qianji.toml")
        );
        assert_eq!(
            paths[1],
            PathBuf::from("/tmp/omni-macro-conf/xiuxian-artisan-workshop/qianji.toml")
        );
        assert_eq!(paths[2], PathBuf::from("/tmp/custom/qianji.toml"));
    }
}

mod test_llm_env_macros {
    use std::process::Command;
    use xiuxian_macros::{env_non_empty, string_first_non_empty};

    fn run_child_test(test_name: &str, envs: &[(&str, &str)]) {
        let current_exe = std::env::current_exe()
            .unwrap_or_else(|error| panic!("resolve current xiuxian-macros test binary: {error}"));
        let output = Command::new(current_exe)
            .arg("--exact")
            .arg(test_name)
            .arg("--ignored")
            .arg("--nocapture")
            .envs(envs.iter().copied())
            .output()
            .unwrap_or_else(|error| panic!("run child xiuxian-macros test {test_name}: {error}"));

        assert!(
            output.status.success(),
            "child test {test_name} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    #[test]
    fn test_env_non_empty_trims_value() {
        run_child_test(
            "test_llm_env_macros::child_env_non_empty_trims_value",
            &[("OMNI_MACROS_TEST_KEY", "  test-key  ")],
        );
    }

    #[test]
    #[ignore = "spawned by test_env_non_empty_trims_value with isolated env"]
    fn child_env_non_empty_trims_value() {
        let value = env_non_empty!("OMNI_MACROS_TEST_KEY");
        assert_eq!(value.as_deref(), Some("test-key"));
    }

    #[test]
    fn test_string_first_non_empty_prefers_first_non_blank() {
        let value = string_first_non_empty!(
            None::<&str>,
            Some(""),
            Some("   "),
            Some("winner"),
            Some("later")
        );
        assert_eq!(value, "winner");
    }
}
