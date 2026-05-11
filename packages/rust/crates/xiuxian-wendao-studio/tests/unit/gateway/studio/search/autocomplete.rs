use super::{
    assert_studio_json_snapshot, build_autocomplete_response, json, make_state_with_docs,
    publish_local_symbol_index,
};

#[tokio::test]
async fn autocomplete_limits_and_filters_prefix() {
    let fixture = make_state_with_docs(vec![
        (
            "doc.md",
            "# Search Design\n\nThis doc starts with Search and discusses Search.\n",
        ),
        ("note.md", "# Search Notes\n\nTaggable text.\n"),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "se", 2).await;

    let Ok(response) = result else {
        panic!("expected autocomplete request to succeed");
    };

    assert_studio_json_snapshot(
        "search_autocomplete_payload",
        json!({
            "prefix": response.prefix,
            "suggestions": response.suggestions.into_iter().map(|suggestion| {
                json!({
                    "text": suggestion.text,
                    "suggestionType": suggestion.suggestion_type,
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn autocomplete_includes_code_symbols() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        ),
        (
            "packages/python/demo/tool.py",
            "class AlphaClient:\n    pass\n\ndef alpha_helper():\n    return None\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "al", 10).await;

    let Ok(response) = result else {
        panic!("expected code-symbol autocomplete request to succeed");
    };

    let suggestions = response
        .suggestions
        .into_iter()
        .map(|suggestion| (suggestion.text, suggestion.suggestion_type.to_string()))
        .collect::<Vec<_>>();

    assert_eq!(
        suggestions,
        vec![
            ("AlphaClient".to_string(), "symbol".to_string()),
            ("AlphaService".to_string(), "symbol".to_string()),
            ("alpha_handler".to_string(), "symbol".to_string()),
            ("alpha_helper".to_string(), "symbol".to_string()),
        ]
    );
}

#[tokio::test]
async fn autocomplete_waits_for_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
    )]);

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "al", 10).await;

    let Ok(response) = result else {
        panic!("expected cold-start autocomplete request to succeed");
    };

    assert!(
        response
            .suggestions
            .iter()
            .any(|suggestion| suggestion.text == "AlphaService")
    );
}
