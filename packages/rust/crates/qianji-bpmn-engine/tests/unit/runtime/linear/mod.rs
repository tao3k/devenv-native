use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnInstanceState, DmnDecisionRef, PendingHostWork, PendingHostWorkKind};

mod completion;
mod dmn;
mod error;
mod host;
mod host_loop;
mod message;

struct PendingHostWorkExpectation<'a> {
    kind: PendingHostWorkKind,
    activity_id: &'a str,
    decision: Option<DmnDecisionRef>,
    script_format: Option<&'a str>,
    script_body: Option<&'a str>,
    event_reference: Option<&'a str>,
    event_name: Option<&'a str>,
}

impl<'a> PendingHostWorkExpectation<'a> {
    fn new(kind: PendingHostWorkKind) -> Self {
        Self {
            kind,
            activity_id: "task",
            decision: None,
            script_format: None,
            script_body: None,
            event_reference: None,
            event_name: None,
        }
    }

    fn with_activity_id(mut self, activity_id: &'a str) -> Self {
        self.activity_id = activity_id;
        self
    }

    fn with_decision(mut self, decision: DmnDecisionRef) -> Self {
        self.decision = Some(decision);
        self
    }

    fn with_script(mut self, script_format: Option<&'a str>, script_body: Option<&'a str>) -> Self {
        self.script_format = script_format;
        self.script_body = script_body;
        self
    }

    fn with_event(mut self, event_reference: Option<&'a str>, event_name: Option<&'a str>) -> Self {
        self.event_reference = event_reference;
        self.event_name = event_name;
        self
    }
}

fn assert_single_pending_host_work(
    instance: &BpmnInstanceState,
    expected: PendingHostWorkExpectation<'_>,
) -> PendingHostWork {
    let pending = instance
        .pending_host_work
        .first()
        .cloned()
        .must("pending host work should be stored");
    assert_eq!(
        pending,
        PendingHostWork {
            token_id: instance.active_tokens[0].token_id,
            process_id: Some(instance.process.process_id.to_string()),
            node_index: 1,
            activity_id: Some(expected.activity_id.to_string()),
            kind: expected.kind,
            decision: expected.decision,
            lane: None,
            script_format: expected.script_format.map(str::to_string),
            script_body: expected.script_body.map(str::to_string),
            human_task_form: None,
            human_task_assignment: None,
            task_io: pending.task_io.clone(),
            claim: None,
            event_reference: expected.event_reference.map(str::to_string),
            event_name: expected.event_name.map(str::to_string),
            work_id: None,
        }
    );
    pending
}
