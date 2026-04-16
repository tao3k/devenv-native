use super::*;

#[test]
fn test_extract_id_references() {
    let text = "See [[#intro]] and [[#architecture]] for details.";
    let refs = extract_id_references(text);
    assert_eq!(refs, vec!["#intro", "#architecture"]);
}

#[test]
fn test_extract_id_references_no_match() {
    let text = "No wiki links here, just [[regular-link]] text.";
    let refs = extract_id_references(text);
    assert!(refs.is_empty());
}

#[test]
fn test_extract_hash_references_with_hash() {
    let text = "See [[#arch-v1@abc123]] for the architecture.";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target_id, "arch-v1");
    assert_eq!(refs[0].expect_hash, Some("abc123".to_string()));
}

#[test]
fn test_extract_hash_references_without_hash() {
    let text = "See [[#intro]] for the introduction.";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target_id, "intro");
    assert_eq!(refs[0].expect_hash, None);
}

#[test]
fn test_extract_hash_references_mixed() {
    let text = "See [[#arch-v1@abc123]] and [[#intro]] and [[#config@def456]].";
    let refs = extract_hash_references(text);
    assert_eq!(refs.len(), 3);
    assert_eq!(refs[0].target_id, "arch-v1");
    assert_eq!(refs[0].expect_hash, Some("abc123".to_string()));
    assert_eq!(refs[1].target_id, "intro");
    assert_eq!(refs[1].expect_hash, None);
    assert_eq!(refs[2].target_id, "config");
    assert_eq!(refs[2].expect_hash, Some("def456".to_string()));
}

#[test]
fn test_extract_hash_references_empty() {
    let text = "No hash-annotated references here.";
    let refs = extract_hash_references(text);
    assert!(refs.is_empty());
}
