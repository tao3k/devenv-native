use super::*;

#[test]
fn run_show_contract_command_renders_wendao_docs_contract_snapshot() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Contract("wendao.docs.navigation".to_string()),
        }),
        "show contract command should render Wendao docs contract snapshot",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Contract"));
    assert!(output.rendered.contains("Name: wendao.docs.navigation"));
    assert!(
        output
            .rendered
            .contains("Kind: wendao-docs-invocation-contract")
    );
    assert!(output.rendered.contains("## Contract TOML"));
    assert!(output.rendered.contains("task_types = ["));
    assert!(output.rendered.contains("path = \"/api/docs/navigation\""));
    assert!(output.rendered.contains("## Schema JSON"));
    assert!(
        output
            .rendered
            .contains("\"title\": \"DocsNavigationToolArgs\"")
    );
    assert!(output.rendered.contains("\"page_id\""));
}
