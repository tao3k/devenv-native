use xiuxian_wendao::analyzers::RegisteredRepository;

use super::repository_uses_managed_remote_source;

#[test]
fn repository_uses_managed_remote_source_only_when_url_is_present() {
    let local_repository = RegisteredRepository::default();
    assert!(!repository_uses_managed_remote_source(&local_repository));

    let remote_repository = RegisteredRepository {
        url: Some("https://example.com/repo.git".to_string()),
        ..RegisteredRepository::default()
    };
    assert!(repository_uses_managed_remote_source(&remote_repository));
}
