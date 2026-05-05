use crate::studio::router::GatewayState;
use xiuxian_wendao::search::SearchCorpusKind;

pub(crate) async fn repo_entity_publication_ready(
    state: &std::sync::Arc<GatewayState>,
    repo_id: &str,
) -> bool {
    state
        .studio
        .search_plane
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, repo_id)
        .await
        .and_then(|record| record.publication)
        .is_some()
}
