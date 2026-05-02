use crate::search::service::tests::support::{derive_status_reason, some_or_panic};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchPlanePhase,
};

#[test]
fn derive_status_reason_marks_failed_refresh_as_retryable_warning() {
    let mut status = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    status.phase = SearchPlanePhase::Failed;
    status.active_epoch = Some(7);
    status.row_count = Some(12);
    status.last_error = Some("builder crashed".to_string());

    let reason = some_or_panic(derive_status_reason(&status), "status reason should exist");

    assert_eq!(reason.code, SearchCorpusStatusReasonCode::BuildFailed);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Warning);
    assert_eq!(reason.action, SearchCorpusStatusAction::RetryBuild);
    assert!(reason.readable);
}
