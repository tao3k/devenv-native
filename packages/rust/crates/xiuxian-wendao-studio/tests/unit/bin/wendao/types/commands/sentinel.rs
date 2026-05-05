use super::{SentinelArgs, SentinelCommand, SentinelWatchArgs, sentinel};
use crate::bin_support::wendao::types::Command;

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
fn test_sentinel_command_creation() {
    let watch_args = SentinelWatchArgs {
        paths: vec!["/watch/path".to_string()],
        debounce_ms: 2000,
    };
    let args = SentinelArgs {
        command: SentinelCommand::Watch(watch_args),
    };
    let cmd = sentinel(&args);
    match cmd {
        Command::Sentinel(sa) => match &sa.command {
            SentinelCommand::Watch(wa) => {
                assert_eq!(wa.paths.len(), 1);
                assert_eq!(wa.paths[0], "/watch/path");
                assert_eq!(wa.debounce_ms, 2000);
            }
        },
        _ => panic!("Expected Sentinel command"),
    }
}

#[test]
fn test_sentinel_args_clone() {
    let args = SentinelArgs {
        command: SentinelCommand::Watch(SentinelWatchArgs {
            paths: vec!["/path".to_string()],
            debounce_ms: 1500,
        }),
    };
    let cloned = args.clone();
    assert!(matches!(cloned.command, SentinelCommand::Watch(_)));
}

#[test]
fn test_sentinel_args_debug() {
    let args = SentinelArgs {
        command: SentinelCommand::Watch(SentinelWatchArgs {
            paths: vec!["/test".to_string()],
            debounce_ms: 1000,
        }),
    };
    let debug_str = format!("{args:?}");
    assert!(debug_str.contains("/test"));
    assert!(debug_str.contains("1000"));
}

#[test]
fn test_sentinel_watch_args_empty_paths() {
    let args = SentinelWatchArgs {
        paths: vec![],
        debounce_ms: 1000,
    };
    let sentinel_args = SentinelArgs {
        command: SentinelCommand::Watch(args),
    };
    match &sentinel_args.command {
        SentinelCommand::Watch(wa) => assert!(wa.paths.is_empty()),
    }
}

#[test]
fn test_sentinel_watch_args_multiple_paths() {
    let args = SentinelWatchArgs {
        paths: vec!["src".to_string(), "lib".to_string(), "tests".to_string()],
        debounce_ms: 1000,
    };
    let sentinel_args = SentinelArgs {
        command: SentinelCommand::Watch(args),
    };
    match &sentinel_args.command {
        SentinelCommand::Watch(wa) => {
            assert_eq!(wa.paths.len(), 3);
            assert_eq!(wa.paths[0], "src");
            assert_eq!(wa.paths[1], "lib");
            assert_eq!(wa.paths[2], "tests");
        }
    }
}
