mod ast;
mod batch;
mod buffered;
mod dispatch;
mod entity;
mod orchestration;
mod search;

pub use self::ast::repository_generic_ast_lang_for_path;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use self::ast::{RepoAstAnalysisIndex, build_repo_ast_analysis_index_from_checkout};
pub use self::buffered::RepoSearchResultLimits;
#[cfg(any(test, feature = "test-support"))]
pub use self::dispatch::{collect_repo_search_targets, repo_search_parallelism};
pub use self::entity::search_repo_entity_hits_for_query;
pub use self::orchestration::RepoCodeSearchExecutionError;
pub use self::orchestration::search_repo_code_outcome_for_query;
pub use self::orchestration::search_repo_intent_outcome;
pub use self::search::search_repo_content_batch;
pub use self::search::search_repo_content_batch_with_repository;
pub use self::search::search_repo_content_hits_for_query;
