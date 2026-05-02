use super::{
    ContractFeedbackCliCommand, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, DirCliCommand,
    MaterializeCliTarget, PathBuf, ShowCliTarget, must_ok, must_some,
    parse_contract_feedback_command, parse_dir_command, to_args,
};

#[test]
fn parse_rest_docs_contract_feedback_command_uses_defaults() {
    let command = must_some(
        must_ok(
            parse_contract_feedback_command(&to_args(&[
                "qianji",
                "contract-feedback",
                "rest-docs",
                "specs/openapi.yaml",
            ])),
            "contract-feedback parse should succeed",
        ),
        "command should be detected",
    );

    let ContractFeedbackCliCommand::RestDocs(command) = command;
    assert_eq!(command.openapi_path, PathBuf::from("specs/openapi.yaml"));
    assert_eq!(command.table_name, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME);
    assert!(!command.no_persist);
    assert!(!command.live_advisory);
    assert!(command.roles.is_empty());
}

#[test]
fn parse_rest_docs_contract_feedback_command_supports_advisory_flags() {
    let command = must_some(
        must_ok(
            parse_contract_feedback_command(&to_args(&[
                "qianji",
                "contract-feedback",
                "rest-docs",
                "specs/openapi.yaml",
                "--workspace-root",
                "/tmp/workspace",
                "--storage-path",
                ".cache/wendao",
                "--table-name",
                "contract_audit",
                "--role",
                "strict_teacher",
                "--role",
                "rest_contract_auditor",
                "--live-advisory",
                "--temperature",
                "0.2",
                "--cognitive-threshold",
                "0.35",
            ])),
            "contract-feedback parse should succeed",
        ),
        "command should be detected",
    );

    let ContractFeedbackCliCommand::RestDocs(command) = command;
    assert_eq!(
        command.workspace_root,
        Some(PathBuf::from("/tmp/workspace"))
    );
    assert_eq!(command.storage_path, Some(PathBuf::from(".cache/wendao")));
    assert_eq!(command.table_name, "contract_audit");
    assert_eq!(
        command.roles,
        vec![
            "strict_teacher".to_string(),
            "rest_contract_auditor".to_string()
        ]
    );
    assert!(command.live_advisory);
    assert_eq!(command.temperature, Some(0.2));
    assert_eq!(command.cognitive_early_halt_threshold, Some(0.35));
}

#[test]
fn parse_show_workdir_command_requires_dir_flag() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&["qianji", "show", "--dir", "/tmp/workdir"])),
            "show parse should succeed",
        ),
        "show command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Show {
            target: ShowCliTarget::Dir(PathBuf::from("/tmp/workdir"))
        }
    );
}

#[test]
fn parse_show_graph_command_requires_graph_flag() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&[
                "qianji",
                "show",
                "--graph",
                "./qianji-flowhub/plan/codex-plan.mmd",
            ])),
            "show graph parse should succeed",
        ),
        "show graph command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Show {
            target: ShowCliTarget::Graph(PathBuf::from("./qianji-flowhub/plan/codex-plan.mmd"))
        }
    );
}

#[test]
fn parse_show_contract_target() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&[
                "qianji",
                "show",
                "--contract",
                "wendao.docs.navigation",
            ])),
            "show contract parse should succeed",
        ),
        "show contract command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Show {
            target: ShowCliTarget::Contract("wendao.docs.navigation".to_string())
        }
    );
}

#[test]
fn parse_check_workdir_command_requires_dir_flag() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&["qianji", "check", "--dir", "demo"])),
            "check parse should succeed",
        ),
        "check command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Check {
            dir: PathBuf::from("demo")
        }
    );
}

#[test]
fn parse_materialize_anchored_command_requires_anchor_scenario_and_dir() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&[
                "qianji",
                "materialize",
                "--anchor",
                "./qianji-flowhub/research/paper/qianji.toml",
                "--scenario",
                "deep_read",
                "--dir",
                "runs/run_001",
            ])),
            "materialize parse should succeed",
        ),
        "materialize command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Materialize {
            target: MaterializeCliTarget::AnchoredScenario {
                anchor: PathBuf::from("./qianji-flowhub/research/paper/qianji.toml"),
                scenario: "deep_read".to_string(),
                dir: PathBuf::from("runs/run_001"),
                current_node: None,
            }
        }
    );
}

#[test]
fn parse_materialize_anchored_command_accepts_current_node() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&[
                "qianji",
                "materialize",
                "--anchor",
                "./qianji-flowhub/research/paper/qianji.toml",
                "--scenario",
                "deep_read",
                "--dir",
                "runs/run_003",
                "--current-node",
                "claim_extract",
            ])),
            "materialize parse with current node should succeed",
        ),
        "materialize command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Materialize {
            target: MaterializeCliTarget::AnchoredScenario {
                anchor: PathBuf::from("./qianji-flowhub/research/paper/qianji.toml"),
                scenario: "deep_read".to_string(),
                dir: PathBuf::from("runs/run_003"),
                current_node: Some("claim_extract".to_string()),
            }
        }
    );
}

#[test]
fn parse_advance_command_requires_dir_and_target_node() {
    let command = must_some(
        must_ok(
            parse_dir_command(&to_args(&[
                "qianji",
                "advance",
                "--dir",
                "runs/run_005",
                "--to",
                "evidence_ground",
            ])),
            "advance parse should succeed",
        ),
        "advance command should be detected",
    );

    assert_eq!(
        command,
        DirCliCommand::Advance {
            dir: PathBuf::from("runs/run_005"),
            to: "evidence_ground".to_string(),
        }
    );
}
