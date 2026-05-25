/// Shared strict-clippy test assertions that avoid `expect`/`expect_err`.
pub(crate) trait MustExt {
    /// Successful value returned by `must`.
    type Output;
    /// Failure value returned by `must_err`.
    type Failure;

    /// Returns the success payload or panics with context.
    fn must(self, context: &str) -> Self::Output;
    /// Returns the failure payload or panics with context.
    fn must_err(self, context: &str) -> Self::Failure;
}

impl<T, E> MustExt for Result<T, E>
where
    E: std::fmt::Debug,
{
    type Output = T;
    type Failure = E;

    fn must(self, context: &str) -> Self::Output {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }

    fn must_err(self, context: &str) -> Self::Failure {
        match self {
            Ok(_) => panic!("{context}: got Ok(..)"),
            Err(error) => error,
        }
    }
}

impl<T> MustExt for Option<T> {
    type Output = T;
    type Failure = ();

    fn must(self, context: &str) -> Self::Output {
        match self {
            Some(value) => value,
            None => panic!("{context}: got None"),
        }
    }

    fn must_err(self, context: &str) -> Self::Failure {
        assert!(self.is_none(), "{context}: got Some(..)");
    }
}

pub(crate) fn apply_pending_host_work_result(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &mut xiuxian_qianji_bpmn_engine::BpmnInstanceState,
    token_id: u64,
    result: xiuxian_qianji_bpmn_engine::PendingHostWorkResult,
    completed_at_ms: u64,
) -> std::result::Result<
    xiuxian_qianji_bpmn_engine::BpmnAdvanceOutcome,
    xiuxian_qianji_bpmn_engine::BpmnEngineError,
> {
    xiuxian_qianji_bpmn_engine::apply_pending_host_work_result(
        xiuxian_qianji_bpmn_engine::PendingHostWorkApplyInput {
            package,
            instance,
            token_id: token_id.into(),
            result,
            completed_at_ms: completed_at_ms.into(),
        },
    )
}

pub(crate) fn claim_request(
    token_id: u64,
    process_id: impl Into<String>,
    activity_id: impl Into<String>,
    claimant: impl Into<String>,
    claimed_at_ms: u64,
) -> xiuxian_qianji_bpmn_engine::PendingHumanTaskClaimRequest {
    xiuxian_qianji_bpmn_engine::PendingHumanTaskClaimRequest::from_input(
        xiuxian_qianji_bpmn_engine::PendingHumanTaskClaimInput {
            token_id: token_id.into(),
            process_id: process_id.into().into(),
            activity_id: activity_id.into().into(),
            claimant: claimant.into(),
            claimed_at_ms: claimed_at_ms.into(),
        },
    )
}

pub(crate) fn release_request(
    token_id: u64,
    process_id: impl Into<String>,
    activity_id: impl Into<String>,
    claimant: impl Into<String>,
    released_at_ms: u64,
) -> xiuxian_qianji_bpmn_engine::PendingHumanTaskReleaseRequest {
    xiuxian_qianji_bpmn_engine::PendingHumanTaskReleaseRequest::from_input(
        xiuxian_qianji_bpmn_engine::PendingHumanTaskReleaseInput {
            token_id: token_id.into(),
            process_id: process_id.into().into(),
            activity_id: activity_id.into().into(),
            claimant: claimant.into(),
            released_at_ms: released_at_ms.into(),
        },
    )
}

pub(crate) fn data_object_io_bpmn() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_service_task_data_object_io">
  <bpmn:process id="service_task_data_object_io" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="enrich">
      <bpmn:ioSpecification>
        <bpmn:dataInput id="Input_Order" name="order"/>
        <bpmn:dataOutput id="Output_Decision" name="decision"/>
        <bpmn:inputSet>
          <bpmn:dataInputRefs>Input_Order</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet>
          <bpmn:dataOutputRefs>Output_Decision</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>OrderRef</bpmn:sourceRef>
        <bpmn:targetRef>Input_Order</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>Output_Decision</bpmn:sourceRef>
        <bpmn:targetRef>OrderRef</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:dataObject id="OrderData" name="Order data"/>
    <bpmn:dataObjectReference id="OrderRef" name="Order ref" dataObjectRef="OrderData"/>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="enrich" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="enrich" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
}
