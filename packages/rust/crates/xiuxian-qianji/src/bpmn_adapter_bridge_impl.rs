use xiuxian_qianji_bpmn_engine::{
    BpmnHostBridge, BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome,
    EventPollRequest, HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ScriptTaskOutcome,
    ScriptTaskRequest, SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest,
    UserTaskOutcome, UserTaskRequest,
};

use super::api::QianjiBpmnHostBridge;

#[async_trait::async_trait]
impl BpmnHostBridge for QianjiBpmnHostBridge {
    async fn dispatch_send_task(
        &self,
        request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        (self.send_task)(request).await
    }

    async fn dispatch_service_task(
        &self,
        request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        (self.service_task)(request).await
    }

    async fn dispatch_script_task(
        &self,
        request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
        (self.script_task)(request).await
    }

    async fn dispatch_user_task(
        &self,
        request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        (self.user_task)(request).await
    }

    async fn dispatch_manual_task(
        &self,
        request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        (self.manual_task)(request).await
    }

    async fn dispatch_business_rule_task(
        &self,
        request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        (self.business_rule_task)(request).await
    }

    async fn poll_external_event(
        &self,
        request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        (self.event_poll)(request).await
    }

    fn now_unix_ms(&self) -> u64 {
        (self.now_unix_ms)()
    }
}
