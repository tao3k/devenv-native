use std::process::Command;

use crate::orgize_runtime::support::tempdir_or_panic;

#[test]
fn standalone_orgize_task_list_json_outputs_limited_recovery_rows() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO First active task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: first-active\n",
            ":NEXT_ACTION: Continue first\n",
            ":RESUME_QUERY: wendao-client orgize task-list --text 'First active'\n",
            ":CLAIM: Org properties feed typed memory recall.\n",
            ":END:\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which preference or naming correction should future generated plans preserve? | Prefer orgid-show for exact recovery. |\n",
            "* TODO Second active task :agent:performance:\n",
            "SCHEDULED: <2026-05-18 Mon ++1d>\n",
            ":PROPERTIES:\n",
            ":ID: second-active\n",
            ":END:\n",
            "* TODO Third active task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: third-active\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("task-list")
        .arg("--limit")
        .arg("2")
        .arg("agenda.org")
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list json: {error}"));
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));
    assert_eq!(parsed["command"], "orgize task-list");
    assert_eq!(parsed["backend"], "duckdb");
    assert_eq!(parsed["rows"], 3);
    assert_eq!(parsed["showing"], 2);
    assert_eq!(parsed["active"], 3);
    assert_eq!(parsed["tasks"].as_array().map_or(0, Vec::len), 2);
    assert_eq!(parsed["tasks"][0]["orgid"], "first-active");
    assert_eq!(parsed["tasks"][0]["title"], "First active task");
    assert_eq!(parsed["tasks"][0]["next"], "Continue first");
    assert_eq!(
        parsed["tasks"][0]["resume"],
        "wendao-client orgize task-list --text 'First active'"
    );
    assert_eq!(parsed["tasks"][0]["memoryObjects"][0]["kind"], "claim");
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][0]["sourceKind"],
        "property"
    );
    assert_eq!(parsed["tasks"][0]["memoryObjects"][0]["sourceKey"], "CLAIM");
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][0]["facet"],
        "memory-claim"
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][0]["value"],
        "Org properties feed typed memory recall."
    );
    assert_eq!(parsed["tasks"][0]["memoryObjects"][1]["kind"], "preference");
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][1]["sourceKind"],
        "reflection"
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][1]["sourceKey"],
        "Which preference or naming correction should future generated plans preserve?"
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][1]["facet"],
        "memory-preference"
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"][1]["value"],
        "Prefer orgid-show for exact recovery."
    );
    assert_eq!(parsed["tasks"][1]["title"], "Second active task");
    assert_eq!(parsed["tasks"][1]["repeat"][0], "scheduled ++1d (catchUp)");
}

#[test]
fn standalone_orgize_task_list_json_renders_serverless_recall_packet() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    write_serverless_recall_packet_agenda(&agenda);

    let output = run_task_list_json(
        temp.path(),
        &["--include-done", "--text", "compact session"],
    );
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));

    assert_eq!(parsed["tasks"].as_array().map_or(0, Vec::len), 2);
    assert_eq!(parsed["tasks"][0]["orgid"], "accepted-memory-sample");
    assert_eq!(
        parsed["tasks"][0]["locator"]["section"]["orgid"],
        "accepted-memory-sample"
    );
    assert_eq!(parsed["tasks"][0]["sourceRangeStart"], 0);
    assert!(
        parsed["tasks"][0]["sourceRangeEnd"]
            .as_u64()
            .is_some_and(|end| end > 0),
        "sourceRangeEnd should be a byte-offset range end: {}",
        parsed["tasks"][0]["sourceRangeEnd"]
    );
    assert_eq!(
        parsed["tasks"][0]["memoryObjects"]
            .as_array()
            .map_or(0, Vec::len),
        2
    );
    assert_eq!(parsed["tasks"][1]["orgid"], "superseded-memory-sample");
    assert_eq!(
        parsed["tasks"][1]["memoryObjects"]
            .as_array()
            .map_or(0, Vec::len),
        0
    );
    assert_eq!(
        parsed["recallPacket"]["schema"],
        "xiuxian_wendao.serverless_memory_recall_packet.v1"
    );
    assert_eq!(
        parsed["recallPacket"]["transport"],
        "local-duckdb-arrow-ready"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"]
            .as_array()
            .map_or(0, Vec::len),
        1
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["orgid"],
        "accepted-memory-sample"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["locator"]["section"]["orgid"],
        "accepted-memory-sample"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["memoryObjects"][1]["locator"]["object"]["sourceKey"],
        "REUSABLE_KNOWLEDGE"
    );
    assert_eq!(parsed["recallPacket"]["rows"][0]["sourceRangeStart"], 0);
    assert!(
        parsed["recallPacket"]["rows"][0]["sourceRangeEnd"]
            .as_u64()
            .is_some_and(|end| end > 0),
        "recall sourceRangeEnd should be a byte-offset range end: {}",
        parsed["recallPacket"]["rows"][0]["sourceRangeEnd"]
    );
}

#[test]
fn standalone_orgize_task_list_json_renders_matched_org_elements() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* DONE Org elements recall sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: org-elements-recall-sample\n",
            ":REUSABLE_KNOWLEDGE: org-elements SQL recall typed facts stay in memoryObjects only.\n",
            ":END:\n",
            "This paragraph is only available through org-elements SQL recall.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_task_list_json(
        temp.path(),
        &["--include-done", "--text", "org-elements SQL recall"],
    );
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));

    assert_eq!(parsed["rows"], 1);
    assert!(
        parsed["snapshotOrgElements"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "read-model should materialize org-elements for locator-backed recall: {parsed}",
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["orgid"],
        "org-elements-recall-sample"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"][0]["kind"],
        "paragraph"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"][0]["locator"]["orgElement"]["type"],
        "paragraph"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"][0]["locator"]["orgElement"]["query"]
            ["table"],
        "agent_org_elements"
    );
    assert!(
        parsed["recallPacket"]["rows"][0]["memoryObjects"][0]["value"]
            .as_str()
            .is_some_and(|value| value.contains("org-elements SQL recall typed facts")),
        "the property-derived typed memory object should still be present: {}",
        parsed["recallPacket"]["rows"][0]["memoryObjects"][0]["value"],
    );
    let matched_elements = parsed["recallPacket"]["rows"][0]["matchedOrgElements"]
        .as_array()
        .unwrap_or_else(|| panic!("matched org elements should be an array"));
    assert!(
        matched_elements
            .iter()
            .all(|element| element["category"] != "property"
                && element["locator"]["orgElement"]["context"] != "propertyDrawer"),
        "matchedOrgElements must stay body/section evidence, not property drawer rows: {matched_elements:?}",
    );
    assert!(
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"][0]["sourceRaw"]
            .as_str()
            .is_some_and(|raw| raw.contains("only available through org-elements SQL recall")),
        "matched org-element sourceRaw should carry the exact structural evidence: {}",
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"][0]["sourceRaw"],
    );
}

#[test]
fn standalone_orgize_task_list_json_matches_properties_without_returning_them_as_org_elements() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("agenda.org");
    std::fs::write(
        &agenda,
        concat!(
            "* DONE Property recall sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: property-recall-sample\n",
            ":REUSABLE_KNOWLEDGE: property-only sentinel selects this memory fact.\n",
            ":END:\n",
            "This body paragraph intentionally does not contain the property-only query text.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_task_list_json(
        temp.path(),
        &["--include-done", "--text", "property-only sentinel"],
    );
    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    let stderr =
        String::from_utf8(output.stderr).unwrap_or_else(|error| panic!("stderr utf8: {error}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("parse json: {error}"));

    assert_eq!(parsed["rows"], 1);
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["orgid"],
        "property-recall-sample"
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["memoryObjects"][0]["value"],
        "property-only sentinel selects this memory fact."
    );
    assert_eq!(
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"]
            .as_array()
            .map_or(usize::MAX, Vec::len),
        0,
        "property matches should select the row but stay out of matchedOrgElements: {}",
        parsed["recallPacket"]["rows"][0]["matchedOrgElements"],
    );
}

fn write_serverless_recall_packet_agenda(agenda: &std::path::Path) {
    std::fs::write(
        agenda,
        concat!(
            "* DONE Accepted memory sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: accepted-memory-sample\n",
            ":TASK_OUTCOME: success\n",
            ":REUSABLE_KNOWLEDGE: The serverless recall path emits compact session packets.\n",
            ":END:\n",
            "* DONE Superseded memory sample :agent:memory:\n",
            "CLOSED: [2026-05-25 Mon]\n",
            ":PROPERTIES:\n",
            ":ID: superseded-memory-sample\n",
            ":MEMORY_STATUS: superseded\n",
            ":TASK_OUTCOME: success\n",
            ":REUSABLE_KNOWLEDGE: A stale compact session packet claim must stay out of recall packets.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn run_task_list_json(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command
        .env_remove("PRJ_CACHE_HOME")
        .arg("--root")
        .arg(root)
        .arg("--output")
        .arg("json")
        .arg("orgize")
        .arg("task-list");
    command.args(args).arg("agenda.org");
    command
        .output()
        .unwrap_or_else(|error| panic!("run orgize task-list json: {error}"))
}
