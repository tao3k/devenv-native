mod native_routes;
mod payload;

use crate::gateway::studio::router::tests::{repo_project, studio_with_repo_projects};
use crate::gateway::studio::router::{configured_repositories, configured_repository};

#[test]
fn resolve_code_ast_repository_and_path_infers_repo_from_prefixed_path() {
    use crate::gateway::studio::router::resolve_code_ast_repository_and_path;

    let studio = studio_with_repo_projects(vec![repo_project("sciml"), repo_project("mcl")]);
    let repositories = configured_repositories(&studio);
    let (repository, path) =
        resolve_code_ast_repository_and_path(&repositories, None, "sciml/src/BaseModelica.jl")
            .unwrap_or_else(|error| {
                panic!("repo should be inferred from prefixed path: {error:?}")
            });
    assert_eq!(repository.id, "sciml");
    assert_eq!(path, "src/BaseModelica.jl");
}

#[test]
fn resolve_code_ast_repository_and_path_strips_explicit_repo_prefix() {
    use crate::gateway::studio::router::resolve_code_ast_repository_and_path;

    let studio = studio_with_repo_projects(vec![repo_project("kernel")]);
    let repositories = configured_repositories(&studio);
    let (repository, path) =
        resolve_code_ast_repository_and_path(&repositories, Some("kernel"), "kernel/src/lib.rs")
            .unwrap_or_else(|error| {
                panic!("explicit repo-scoped code AST path should normalize: {error:?}")
            });
    assert_eq!(repository.id, "kernel");
    assert_eq!(path, "src/lib.rs");
}

#[test]
fn resolve_code_ast_repository_and_path_rejects_conflicting_repo_prefix() {
    use crate::gateway::studio::router::resolve_code_ast_repository_and_path;
    use axum::http::StatusCode;

    let studio = studio_with_repo_projects(vec![repo_project("kernel"), repo_project("main")]);
    let repositories = configured_repositories(&studio);
    let Err(error) =
        resolve_code_ast_repository_and_path(&repositories, Some("kernel"), "main/docs/index.md")
    else {
        panic!("conflicting repo-scoped code AST path should fail");
    };
    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "REPO_PATH_MISMATCH");
}

#[test]
fn resolve_code_ast_repository_and_path_requires_repo_when_ambiguous() {
    use crate::gateway::studio::router::resolve_code_ast_repository_and_path;
    use axum::http::StatusCode;

    let studio = studio_with_repo_projects(vec![repo_project("sciml"), repo_project("mcl")]);
    let repositories = configured_repositories(&studio);
    let Err(error) =
        resolve_code_ast_repository_and_path(&repositories, None, "src/BaseModelica.jl")
    else {
        panic!("should fail when repo cannot be inferred");
    };
    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_REPO");
}

#[test]
fn configured_repository_matches_repo_identifier_case_insensitively() {
    let studio = studio_with_repo_projects(vec![repo_project("DifferentialEquations.jl")]);

    let repository = configured_repository(&studio, "differentialequations.jl")
        .unwrap_or_else(|error| panic!("repo lookup should ignore ASCII case: {error:?}"));

    assert_eq!(repository.id, "DifferentialEquations.jl");
}
