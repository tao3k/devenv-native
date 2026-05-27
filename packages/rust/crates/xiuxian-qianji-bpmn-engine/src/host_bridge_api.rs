//! Host-bridge trait definitions.

use crate::host_types_api::{
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ScriptTaskOutcome, ScriptTaskRequest,
    SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, TaskOutcome,
    TaskRequest, UserTaskOutcome, UserTaskRequest,
};

/// Host callback surface implemented by `xiuxian-qianji` or another host.
#[async_trait::async_trait]
pub trait BpmnHostBridge {
    /// Dispatches generic BPMN task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_task(
        &self,
        request: TaskRequest,
    ) -> std::result::Result<TaskOutcome, HostBridgeError>;

    /// Dispatches send-task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_send_task(
        &self,
        request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError>;

    /// Dispatches service-task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_service_task(
        &self,
        request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError>;

    /// Dispatches script-task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_script_task(
        &self,
        request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError>;

    /// Dispatches user-task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_user_task(
        &self,
        request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError>;

    /// Dispatches manual-task work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_manual_task(
        &self,
        request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError>;

    /// Dispatches business-rule work.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot execute the request.
    async fn dispatch_business_rule_task(
        &self,
        request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError>;

    /// Polls or resolves external-event progress.
    ///
    /// # Errors
    ///
    /// Returns [`HostBridgeError`] when the host cannot satisfy the request.
    async fn poll_external_event(
        &self,
        request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError>;

    /// Returns the current Unix timestamp in milliseconds.
    fn now_unix_ms(&self) -> u64;
}
