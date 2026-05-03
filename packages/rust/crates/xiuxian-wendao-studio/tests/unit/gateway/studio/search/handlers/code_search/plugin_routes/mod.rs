use crate::studio::search::handlers::code_search::build_code_search_response;
use crate::studio::search::handlers::tests::linked_parser_summary::{
    ensure_linked_modelica_parser_summary_service, ensure_linked_parser_summary_service,
};
use crate::studio::search::handlers::tests::test_studio_state;
use crate::studio::test_support::{assert_studio_json_snapshot, search_response_snapshot};
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry,
};

mod alias_scope;
mod guardrails;
mod live_plugins;
mod repo_scoped_ast;
mod search_only;
mod support;

use support::{
    create_sample_html_repo, create_sample_julia_repo, create_sample_modelica_repo,
    create_sample_rust_repo, create_sample_toml_repo, load_code_search_response,
    publish_repository_snapshot, repo_code_document,
};
