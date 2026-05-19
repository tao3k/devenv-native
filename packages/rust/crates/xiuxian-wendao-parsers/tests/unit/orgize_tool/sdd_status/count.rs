use xiuxian_wendao_parsers::{OrgizeSddStatusRequest, count_sdd_status_issues};

use crate::orgize_tool::support::tempdir_or_panic;

#[test]
fn count_sdd_status_issues_counts_drift_and_missing_paths() {
    let temp = tempdir_or_panic();
    let clean = temp.path().join("clean.org");
    let drifted = temp.path().join("drifted.org");
    let missing = temp.path().join("missing");
    std::fs::write(
        &clean,
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean status should not count.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));
    std::fs::write(
        &drifted,
        concat!(
            "* Drifted View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write drifted sdd org: {error}"));

    let count = count_sdd_status_issues(&OrgizeSddStatusRequest {
        paths: vec![clean, drifted, missing],
        issues_only: false,
    })
    .unwrap_or_else(|error| panic!("count sdd status issues: {error}"));

    assert_eq!(count, 4);
}
