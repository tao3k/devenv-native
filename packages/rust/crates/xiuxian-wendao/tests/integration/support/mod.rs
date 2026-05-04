use std::process::Command;

#[path = "../../support/linked_parser_summary.rs"]
pub(crate) mod linked_parser_summary;
#[path = "../../support/repo_fixture.rs"]
pub(crate) mod repo_fixture;
#[path = "../../support/repo_intelligence.rs"]
pub(crate) mod repo_intelligence;
#[path = "../../support/repo_parser_summary/mod.rs"]
pub(crate) mod repo_parser_summary;
#[path = "../../support/repo_projection_support.rs"]
pub(crate) mod repo_projection_support;

pub(crate) fn wendao_command() -> Command {
    Command::new(std::env::var_os("CARGO_BIN_EXE_wendao").unwrap_or_else(|| "wendao".into()))
}
