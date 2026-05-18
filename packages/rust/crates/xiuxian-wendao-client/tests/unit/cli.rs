#[cfg(feature = "semantic-sql")]
#[path = "cli/semantic.rs"]
mod semantic;

use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{ClientCli, ClientCommand, GetCommand, LintCommand, OrgizeCommand};

#[test]
fn parses_markdown_lint_command() {
    let cli = ClientCli::parse_from(["wendao", "lint", "markdown", "docs"]);
    let ClientCommand::Lint { command } = cli.command else {
        panic!("expected lint command");
    };
    assert!(matches!(command, LintCommand::Markdown(_)));
}

#[test]
fn parses_get_toc_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "toc", "docs/guides"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => assert_eq!(args.target, PathBuf::from("docs/guides")),
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}

#[test]
fn parses_get_page_index_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "page-index"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::PageIndex(args) => assert_eq!(args.target, PathBuf::from(".")),
        GetCommand::Toc(_) => panic!("expected get page-index command"),
    }
}

#[test]
fn parses_get_toc_file_target_command() {
    let cli = ClientCli::parse_from(["wendao", "get", "toc", "docs/guides/intro.md"]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => assert_eq!(args.target, PathBuf::from("docs/guides/intro.md")),
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}

#[test]
fn parses_get_toc_command_with_repeatable_ignore_dirs() {
    let cli = ClientCli::parse_from([
        "wendao",
        "get",
        "toc",
        "docs",
        "--ignore",
        "generated",
        "--ignore",
        ".cache",
    ]);
    let ClientCommand::Get { command } = cli.command else {
        panic!("expected get command");
    };
    match command {
        GetCommand::Toc(args) => {
            assert_eq!(args.target, PathBuf::from("docs"));
            assert_eq!(
                args.ignore_dirs,
                vec!["generated".to_string(), ".cache".to_string()]
            );
        }
        GetCommand::PageIndex(_) => panic!("expected get toc command"),
    }
}

#[test]
fn parses_orgize_agent_planning_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "agent-planning",
        "--date",
        "2026-05-17",
        ".agent/org/agenda.org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::AgentPlanning(args) => {
            assert_eq!(args.date, "2026-05-17");
            assert_eq!(args.paths, vec![PathBuf::from(".agent/org/agenda.org")]);
        }
        _ => panic!("expected orgize agent-planning command"),
    }
}

#[test]
fn parses_orgize_sparse_tree_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "sparse-tree",
        "--match",
        "+agent",
        "--exclude-done",
        ".agent/org/agenda.org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::SparseTree(args) => {
            assert_eq!(args.match_expression.as_deref(), Some("+agent"));
            assert!(args.visibility.exclude_done);
            assert_eq!(args.paths, vec![PathBuf::from(".agent/org/agenda.org")]);
        }
        _ => panic!("expected orgize sparse-tree command"),
    }
}

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
