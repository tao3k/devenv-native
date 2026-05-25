use crate::runtime::runtime_optional_output_io;
use xiuxian_qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnNodeKind, BpmnNodeSpec,
    BpmnParallelMultiInstanceSpec, BpmnProcessSpec, BpmnRepeatSpec, BpmnTimerKind, BpmnTimerSpec,
    ProcessKey,
};

pub(super) fn parallel_multi_instance_non_interrupting_boundary_process(
    process_id: &str,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::UserTask)
                .with_repeat(BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::new(3),
                ))
                .with_task_io(runtime_optional_output_io()),
            BpmnNodeSpec::new(2, "review_timeout", BpmnNodeKind::BoundaryEvent)
                .with_boundary_attachment(1, false),
            BpmnNodeSpec::new(3, "approved_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(4, "timeout_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 3, None::<&str>),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(2, BpmnEventKind::Timer)
                .with_name("ReviewTimeout")
                .with_timer(BpmnTimerSpec::new(BpmnTimerKind::Duration, "PT30M")),
        ],
    )
}
