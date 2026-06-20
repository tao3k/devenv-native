//! `search::repo_search` owns Wendao search repo search behavior.

mod batch;
mod buffered;
mod content;
mod dispatch;
mod entity;
mod orchestration;

pub use self::batch::{search_repo_content_batch, search_repo_content_batch_with_repository};
pub use self::buffered::RepoSearchResultLimits;
pub use self::content::search_repo_content_hits_for_query;
#[cfg(any(test, feature = "test-support"))]
pub use self::dispatch::{collect_repo_search_targets, repo_search_parallelism};
pub use self::entity::search_repo_entity_hits_for_query;
pub use self::orchestration::RepoCodeSearchExecutionError;
pub use self::orchestration::search_repo_code_outcome_for_query;
pub use self::orchestration::search_repo_intent_outcome;
