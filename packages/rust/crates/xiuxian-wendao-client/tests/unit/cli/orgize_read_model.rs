use super::support::{ClientCli, ClientCommand, OrgizeCommand, Parser, PathBuf};

#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_read_model_command_without_backend_flag() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "read-model",
        ".cache/agent/org/agenda.org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::ReadModel(args) => {
            assert_eq!(
                args.paths,
                vec![PathBuf::from(".cache/agent/org/agenda.org")]
            );
        }
        _ => panic!("expected orgize read-model command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_list_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-list",
        "--text",
        "DuckDB",
        "--tag",
        "achievement",
        "--include-done",
        "--limit",
        "5",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskList(args) => {
            assert_eq!(args.text.as_deref(), Some("DuckDB"));
            assert_eq!(args.tags, vec!["achievement".to_string()]);
            assert!(args.include_done);
            assert_eq!(args.limit, 5);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-list command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_probe_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-probe",
        "--cached",
        "--text",
        "Audio OpenRouter",
        "--limit",
        "2",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskProbe(args) => {
            assert!(args.cached);
            assert_eq!(args.text, "Audio OpenRouter");
            assert_eq!(args.limit, 2);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-probe command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_orgid_show_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "orgid-show",
        "--cached",
        "--id",
        "target-task",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::OrgidShow(args) => {
            assert!(args.cached);
            assert_eq!(args.id, "target-task");
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize orgid-show command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn rejects_legacy_orgize_ogrid_show_command() {
    let result = ClientCli::try_parse_from([
        "wendao",
        "orgize",
        "ogrid-show",
        "--id",
        "target-task",
        ".cache/agent/org",
    ]);

    assert!(
        result.is_err(),
        "legacy ogrid-show must not remain an alias"
    );
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_sdd_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-sdd",
        "--cached",
        "--id",
        "target-task",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskSdd(args) => {
            assert!(args.cached);
            assert_eq!(args.id, "target-task");
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-sdd command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_recover_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-recover",
        "--cached",
        "--text",
        "flowhub",
        "--tag",
        "agent",
        "--limit",
        "5",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskRecover(args) => {
            assert!(args.cached);
            assert_eq!(args.text.as_deref(), Some("flowhub"));
            assert_eq!(args.tags, vec!["agent".to_string()]);
            assert_eq!(args.limit, 5);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-recover command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_report_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-report",
        "--text",
        "Agent",
        "--tag",
        "achievement",
        "--include-archived",
        "--limit",
        "3",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskReport(args) => {
            assert_eq!(args.text.as_deref(), Some("Agent"));
            assert_eq!(args.tags, vec!["achievement".to_string()]);
            assert!(args.include_archived);
            assert_eq!(args.limit, 3);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-report command"),
    }
}
#[cfg(feature = "orgize-agent-read-model")]
#[test]
fn parses_orgize_task_archive_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "task-archive",
        "--apply",
        "--tag",
        "achievement",
        "--limit",
        "2",
        ".cache/agent/org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::TaskArchive(args) => {
            assert!(args.apply);
            assert_eq!(args.tags, vec!["achievement".to_string()]);
            assert_eq!(args.limit, 2);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/org")]);
        }
        _ => panic!("expected orgize task-archive command"),
    }
}
