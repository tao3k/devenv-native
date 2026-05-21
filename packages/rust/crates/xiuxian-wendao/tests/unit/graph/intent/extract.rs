use super::extract_intent;

#[test]
fn extracts_git_target_from_action_context() {
    let intent = extract_intent("please commit staged changes");

    assert_eq!(intent.action.as_deref(), Some("commit"));
    assert_eq!(intent.target.as_deref(), Some("git"));
    assert_eq!(intent.context, vec!["staged", "changes"]);
}

#[test]
fn extracts_explicit_target_and_context_keywords() {
    let intent = extract_intent("search docs for rust harness policy");

    assert_eq!(intent.action.as_deref(), Some("search"));
    assert_eq!(intent.target.as_deref(), Some("docs"));
    assert_eq!(intent.context, vec!["rust", "harness", "policy"]);
}
