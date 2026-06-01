#[path = "../../../../src/bpmn/http/llm_task_documentation.rs"]
mod source;

#[test]
fn extracts_prefixed_activity_documentation() {
    let xml = r#"
        <bpmn:process id="Process_1">
          <bpmn:serviceTask id="Task_Retry" name="Retry init">
            <bpmn:documentation>Output status as &quot;ready&quot; and isReady as true.</bpmn:documentation>
          </bpmn:serviceTask>
        </bpmn:process>
    "#;

    let documentation = source::extract_activity_documentation(xml, "Task_Retry");

    assert_eq!(
        documentation.as_deref(),
        Some("Output status as \"ready\" and isReady as true.")
    );
}

#[test]
fn extracts_documentation_from_file_source_ref() {
    let dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("documentation tempdir should allocate: {error}"));
    let path = dir.path().join("flow.bpmn");
    std::fs::write(
        &path,
        r#"
        <bpmn:process id="Process_1">
          <bpmn:serviceTask id="Task_Retry">
            <bpmn:documentation>Retry from file.</bpmn:documentation>
          </bpmn:serviceTask>
        </bpmn:process>
    "#,
    )
    .unwrap_or_else(|error| panic!("documentation fixture should write: {error}"));
    let source_ref = format!("file://{}", path.display());

    let documentation = source::bpmn_task_documentation(Some(&source_ref), Some("Task_Retry"));

    assert_eq!(documentation.as_deref(), Some("Retry from file."));
}

#[test]
fn ignores_neighbor_activity_documentation() {
    let xml = r#"
        <process id="Process_1">
          <serviceTask id="Task_Init">
            <documentation>Initial documentation.</documentation>
          </serviceTask>
          <serviceTask id="Task_Retry">
            <documentation>Retry documentation.</documentation>
          </serviceTask>
        </process>
    "#;

    let documentation = source::extract_activity_documentation(xml, "Task_Retry");

    assert_eq!(documentation.as_deref(), Some("Retry documentation."));
}
