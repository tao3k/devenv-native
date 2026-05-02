use serde_json::Value;

use fixture_contract_support::{
    SearchDirectivesFixture, assert_search_directives_fixture, search_payload_snapshot,
};

use super::wendao_cmd;

#[test]
fn test_wendao_search_semantic_filter_flags() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = SearchDirectivesFixture::build("semantic_filter_flags")?;

    let mention_output = wendao_cmd()
        .arg("--root")
        .arg(fixture.root())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--mentions-of")
        .arg("Alpha signal")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        mention_output.status.success(),
        "wendao search with mentions-of failed: {}",
        String::from_utf8_lossy(&mention_output.stderr)
    );

    let missing_backlink_output = wendao_cmd()
        .arg("--root")
        .arg(fixture.root())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--missing-backlink")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        missing_backlink_output.status.success(),
        "wendao search with missing-backlink failed: {}",
        String::from_utf8_lossy(&missing_backlink_output.stderr)
    );

    let mention_payload: Value = serde_json::from_str(&String::from_utf8(mention_output.stdout)?)?;
    let missing_backlink_payload: Value =
        serde_json::from_str(&String::from_utf8(missing_backlink_output.stdout)?)?;
    let actual = serde_json::json!({
        "mentions_of": search_payload_snapshot(&mention_payload),
        "missing_backlink": search_payload_snapshot(&missing_backlink_payload),
    });
    assert_search_directives_fixture("semantic_filter_flags", "result.json", &actual);

    Ok(())
}
