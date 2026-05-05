use super::{
    Duration, PathBuf, SentinelArgs, SentinelCommand, SentinelConfig, SentinelWatchArgs,
    parse_paths,
};

#[test]
fn test_sentinel_watch_args_default() {
    let args = SentinelWatchArgs {
        paths: vec![],
        debounce_ms: 1000,
    };
    assert!(args.paths.is_empty());
    assert_eq!(args.debounce_ms, 1000);
}

#[test]
fn test_sentinel_watch_args_with_paths() {
    let args = SentinelWatchArgs {
        paths: vec!["/path/to/watch".to_string()],
        debounce_ms: 500,
    };
    assert_eq!(args.paths.len(), 1);
    assert_eq!(args.debounce_ms, 500);
}

#[test]
fn test_sentinel_watch_args_clone() {
    let args = SentinelWatchArgs {
        paths: vec!["/src".to_string(), "/lib".to_string()],
        debounce_ms: 2000,
    };
    let cloned = args.clone();
    assert_eq!(args.paths, cloned.paths);
    assert_eq!(args.debounce_ms, cloned.debounce_ms);
}

#[test]
fn test_parse_paths_empty() {
    let paths: Vec<String> = vec![];
    let result = parse_paths(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_parse_paths_single() {
    let paths = vec!["/home/user/project".to_string()];
    let result = parse_paths(&paths);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], PathBuf::from("/home/user/project"));
}

#[test]
fn test_parse_paths_multiple() {
    let paths = vec!["/path/one".to_string(), "/path/two".to_string()];
    let result = parse_paths(&paths);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], PathBuf::from("/path/one"));
    assert_eq!(result[1], PathBuf::from("/path/two"));
}

#[test]
fn test_parse_paths_comma_separated() {
    let paths = vec!["/path/one,/path/two,/path/three".to_string()];
    let result = parse_paths(&paths);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], PathBuf::from("/path/one"));
    assert_eq!(result[1], PathBuf::from("/path/two"));
    assert_eq!(result[2], PathBuf::from("/path/three"));
}

#[test]
fn test_parse_paths_mixed() {
    let paths = vec!["/path/one,/path/two".to_string(), "/path/three".to_string()];
    let result = parse_paths(&paths);
    assert_eq!(result.len(), 3);
}

#[test]
fn test_parse_paths_relative() {
    let paths = vec!["src,lib,tests".to_string()];
    let result = parse_paths(&paths);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], PathBuf::from("src"));
    assert_eq!(result[1], PathBuf::from("lib"));
    assert_eq!(result[2], PathBuf::from("tests"));
}

#[test]
fn test_parse_paths_with_trailing_comma() {
    let paths = vec!["/path/one,".to_string()];
    let result = parse_paths(&paths);
    // Trailing comma creates an empty string which becomes an empty PathBuf
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], PathBuf::from("/path/one"));
    assert_eq!(result[1], PathBuf::from(""));
}

#[test]
fn test_debounce_duration() {
    let args = SentinelWatchArgs {
        paths: vec![],
        debounce_ms: 500,
    };
    let duration = Duration::from_millis(args.debounce_ms);
    assert_eq!(duration, Duration::from_millis(500));
}

#[test]
fn test_sentinel_config_creation() {
    let config = SentinelConfig {
        watch_paths: vec![PathBuf::from("src"), PathBuf::from("docs")],
        debounce_duration: Duration::from_millis(500),
    };
    assert_eq!(config.watch_paths.len(), 2);
    assert_eq!(config.debounce_duration, Duration::from_millis(500));
}

#[test]
fn test_sentinel_args_structure() {
    let watch_args = SentinelWatchArgs {
        paths: vec!["src".to_string()],
        debounce_ms: 1000,
    };
    let args = SentinelArgs {
        command: SentinelCommand::Watch(watch_args),
    };
    match &args.command {
        SentinelCommand::Watch(wa) => {
            assert_eq!(wa.paths.len(), 1);
            assert_eq!(wa.debounce_ms, 1000);
        }
    }
}
