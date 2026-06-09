use crate::bpmn::http_transport::llm_completion_shape as source;

#[test]
fn fenced_json_satisfies_multi_output_bindings() {
    let content = "```json\n{\"status\":\"ready\",\"isReady\":true}\n```";
    let shaped = source::shape_llm_content_for_bpmn_outputs(
        content,
        &["status".to_owned(), "isReady".to_owned()],
    );

    assert_eq!(shaped["status"], "ready");
    assert_eq!(shaped["isReady"], true);
}

#[test]
fn multi_output_plain_text_keeps_content_for_precision_gate() {
    let shaped = source::shape_llm_content_for_bpmn_outputs(
        "not json",
        &["status".to_owned(), "isReady".to_owned()],
    );

    assert_eq!(shaped["content"], "not json");
}
