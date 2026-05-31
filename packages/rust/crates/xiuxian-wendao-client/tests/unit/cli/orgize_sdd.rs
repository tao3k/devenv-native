use super::support::{ClientCli, ClientCommand, OrgizeCommand, OrgizeSddCommand, Parser, PathBuf};

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
