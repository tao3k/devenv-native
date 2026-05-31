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
