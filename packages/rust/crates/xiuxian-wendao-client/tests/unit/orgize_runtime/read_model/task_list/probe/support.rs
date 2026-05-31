pub(super) use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

pub(super) fn write_audio_openrouter_probe_agenda(agenda: &std::path::Path) {
    std::fs::write(
        agenda,
        concat!(
            "* TODO Audio OpenRouter gate <2026-05-23 Sat> :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: audio-gate\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-gate\n",
            ":SDD: .cache/agent/sdd/audio.org\n",
            ":NEXT_ACTION: Curate reference rows\n",
            ":END:\n",
            "** Evidence\n",
            "Large body should not render in a memory probe.\n",
            "* TODO Other task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: other-task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}
pub(super) fn assert_audio_openrouter_probe_output(stdout: &str) {
    assert!(
        stdout.contains("title: Audio OpenRouter gate <2026-05-23 Sat>"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("[PROBE"), "stdout: {stdout}");
    assert!(!stdout.contains("backend:"), "stdout: {stdout}");
    assert!(!stdout.contains("database:"), "stdout: {stdout}");
    assert!(!stdout.contains("rows:"), "stdout: {stdout}");
    assert!(!stdout.contains("orgid: audio-gate"), "stdout: {stdout}");
    assert!(!stdout.contains("state: TODO"), "stdout: {stdout}");
    assert!(
        !stdout.contains("file-key: audio_openrouter_lane"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("status: active"), "stdout: {stdout}");
    assert_probe_metadata(stdout);
    assert!(
        stdout.contains("show: wendao-client orgize orgid-show --cached --id audio-gate"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(
            "graph: flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"]"
        ),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("Large body should not render"),
        "stdout: {stdout}"
    );
}
pub(super) fn assert_probe_metadata(stdout: &str) {
    assert!(
        stdout.contains("package: xiuxian-wendao-analyzer"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("slice: audio-openrouter-gate"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("\nsdd:"), "stdout: {stdout}");
    assert!(
        stdout.contains(
            "graph: flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"]"
        ),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("next: Curate reference rows"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("execplan:"), "stdout: {stdout}");
    assert!(!stdout.contains("stable-ref:"), "stdout: {stdout}");
    assert!(!stdout.contains("query-title:"), "stdout: {stdout}");
    assert!(!stdout.contains("query-file:"), "stdout: {stdout}");
}
