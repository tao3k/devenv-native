use crate::search::service::helpers::summarize_issues;
use crate::search::service::tests::support::some_or_panic;
use crate::search::{SearchCorpusIssue, SearchCorpusIssueCode, SearchCorpusIssueFamily};

#[test]
fn summarize_issues_prefers_highest_priority_code_and_marks_mixed_family() {
    let summary = some_or_panic(
        summarize_issues(&[
            SearchCorpusIssue {
                code: SearchCorpusIssueCode::RepoIndexFailed,
                readable: true,
                repo_id: Some("alpha/repo".to_string()),
                current_revision: Some("rev-2".to_string()),
                published_revision: Some("rev-1".to_string()),
                message: "alpha/repo: git fetch failed".to_string(),
            },
            SearchCorpusIssue {
                code: SearchCorpusIssueCode::PublishedManifestMissing,
                readable: false,
                repo_id: Some("beta/repo".to_string()),
                current_revision: Some("rev-9".to_string()),
                published_revision: None,
                message: "beta/repo: published state missing".to_string(),
            },
        ]),
        "summary should exist",
    );

    assert_eq!(summary.family, SearchCorpusIssueFamily::Mixed);
    assert_eq!(
        summary.primary_code,
        SearchCorpusIssueCode::PublishedManifestMissing
    );
    assert_eq!(summary.issue_count, 2);
    assert_eq!(summary.readable_issue_count, 1);
}
