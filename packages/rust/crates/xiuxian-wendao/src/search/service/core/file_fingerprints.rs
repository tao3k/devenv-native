use std::collections::BTreeMap;

use super::types::SearchPlaneService;
use crate::search::SearchFileFingerprint;
use crate::search::cache::SearchPlaneFileFingerprintScope;

impl SearchPlaneService {
    pub(crate) async fn file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
    ) -> BTreeMap<String, SearchFileFingerprint> {
        let fingerprints: BTreeMap<String, SearchFileFingerprint> = self
            .cache
            .get_file_fingerprints(scope)
            .await
            .unwrap_or_default();
        fingerprints
    }

    pub(crate) async fn set_file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
        fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    ) {
        self.cache.set_file_fingerprints(scope, fingerprints).await;
    }

    pub(crate) async fn delete_file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
    ) {
        self.cache.delete_file_fingerprints(scope).await;
    }
}
