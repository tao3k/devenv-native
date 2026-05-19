use xiuxian_wendao_parsers::{OrgizeSddStatusRequest, render_sdd_status_json};

use crate::orgize_tool::support::tempdir_or_panic;

#[test]
fn render_sdd_status_json_uses_stable_contract_shape() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("sdd.org");
    std::fs::write(
        &path,
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Agent planning architecture boundaries.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write json sdd org: {error}"));

    let rendered = render_sdd_status_json(&OrgizeSddStatusRequest {
        paths: vec![path],
        issues_only: false,
    })
    .unwrap_or_else(|error| panic!("render sdd status json: {error}"));
    let payload: serde_json::Value =
        serde_json::from_str(&rendered).unwrap_or_else(|error| panic!("json parse: {error}"));

    assert_eq!(payload["format"], "orgize.sdd.status.v1");
    assert_eq!(payload["files"][0]["exists"], true);
    assert_eq!(payload["files"][0]["architectureNodes"], 1);
    assert_eq!(payload["files"][0]["summary"]["kinds"]["system"], 1);
    assert_eq!(payload["files"][0]["summary"]["statuses"]["review"], 1);
    assert_eq!(payload["files"][0]["summary"]["issues"], 0);
    assert_eq!(payload["files"][0]["nodes"][0]["kind"], "system");
}

#[test]
fn render_sdd_status_json_issues_only_filters_clean_files() {
    let temp = tempdir_or_panic();
    std::fs::write(
        temp.path().join("clean.org"),
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean status should be filtered.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));
    std::fs::write(
        temp.path().join("drifted.org"),
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

    let rendered = render_sdd_status_json(&OrgizeSddStatusRequest {
        paths: vec![temp.path().to_path_buf()],
        issues_only: true,
    })
    .unwrap_or_else(|error| panic!("render issues-only sdd status json: {error}"));
    let payload: serde_json::Value =
        serde_json::from_str(&rendered).unwrap_or_else(|error| panic!("json parse: {error}"));

    assert_eq!(payload["format"], "orgize.sdd.status.v1");
    assert_eq!(payload["files"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["files"][0]["nodes"][0]["title"], "Drifted View");
    assert!(
        payload["files"][0]["summary"]["issues"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
}
