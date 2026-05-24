use super::{check_flowhub, flowhub_root, real_flowhub_fixture_available};

#[test]
fn real_flowhub_uses_org_bpmn_pairs_without_live_mermaid_files() {
    if !real_flowhub_fixture_available() {
        return;
    }

    let root = flowhub_root();
    for removed_mermaid_file in [
        "plan/agent-coding.mmd",
        "wendao/docs-search.mmd",
        "research/paper/paper-canonicalize.mmd",
        "research/paper/paper-deep-read.mmd",
        "research/paper/paper-compare.mmd",
    ] {
        assert!(!root.join(removed_mermaid_file).exists());
    }

    for org_file in [
        "plan/agent-coding.org",
        "wendao/docs-search.org",
        "research/paper/paper-canonicalize.org",
        "research/paper/paper-deep-read.org",
        "research/paper/paper-compare.org",
    ] {
        let source = std::fs::read_to_string(root.join(org_file))
            .unwrap_or_else(|error| panic!("should read {org_file}: {error}"));
        assert!(source.contains(":BPMN_SOURCE:"));
        assert!(source.contains("#+begin_src mermaid"));
        assert!(source.contains("#+end_src"));
    }

    let report = check_flowhub(root)
        .unwrap_or_else(|error| panic!("real Flowhub root should check: {error}"));
    assert!(report.is_valid());
}
