use super::support::{ClientCli, ClientCommand, OrgizeCommand, OrgizeEvalCommand, Parser, PathBuf};

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
