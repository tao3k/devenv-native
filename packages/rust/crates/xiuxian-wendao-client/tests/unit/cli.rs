#[cfg(feature = "semantic-sql")]
#[path = "cli/semantic.rs"]
mod semantic;

use clap::Parser;
use std::path::PathBuf;
use xiuxian_wendao_client::{
    ClientCli, ClientCommand, GetCommand, LintCommand, OrgizeCommand, OrgizeEvalCommand,
    OrgizeSddCommand,
};

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

#[test]
fn parses_orgize_eval_plan_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "eval",
        "plan",
        "--json",
        "verify",
        ".cache/agent/org/task.org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::Eval {
            command: OrgizeEvalCommand::Plan(args),
        } => {
            assert!(args.json);
            assert_eq!(args.name, "verify");
            assert_eq!(args.path, PathBuf::from(".cache/agent/org/task.org"));
        }
        _ => panic!("expected orgize eval plan command"),
    }
}

#[test]
fn parses_orgize_eval_patch_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "eval",
        "patch",
        "--write",
        "--stdout-file",
        ".cache/agent/evidence/result.txt",
        "--exit-code",
        "0",
        "verify",
        ".cache/agent/org/task.org",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::Eval {
            command: OrgizeEvalCommand::Patch(args),
        } => {
            assert!(args.write);
            assert_eq!(
                args.stdout_file,
                Some(PathBuf::from(".cache/agent/evidence/result.txt"))
            );
            assert_eq!(args.exit_code, Some(0));
            assert_eq!(args.name, "verify");
            assert_eq!(args.path, PathBuf::from(".cache/agent/org/task.org"));
        }
        _ => panic!("expected orgize eval patch command"),
    }
}

#[test]
fn parses_orgize_sdd_status_json_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "sdd",
        "status",
        "--json",
        "--issues-only",
        "--fail-on-issues",
        ".cache/agent/sdd",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::Sdd {
            command: OrgizeSddCommand::Status(args),
        } => {
            assert!(args.json);
            assert!(args.issues_only);
            assert!(args.fail_on_issues);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/sdd")]);
        }
        _ => panic!("expected orgize sdd status command"),
    }
}

#[test]
fn parses_orgize_sdd_graph_diff_command() {
    let cli = ClientCli::parse_from([
        "wendao",
        "orgize",
        "sdd",
        "graph-diff",
        "--fail-on-drift",
        ".cache/agent/sdd",
    ]);
    let ClientCommand::Orgize { command } = cli.command else {
        panic!("expected orgize command");
    };
    match command {
        OrgizeCommand::Sdd {
            command: OrgizeSddCommand::GraphDiff(args),
        } => {
            assert!(args.fail_on_drift);
            assert_eq!(args.paths, vec![PathBuf::from(".cache/agent/sdd")]);
        }
        _ => panic!("expected orgize sdd graph-diff command"),
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
