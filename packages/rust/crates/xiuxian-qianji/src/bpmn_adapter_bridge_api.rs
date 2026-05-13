//! Bpmn adapter bridge api surface for `xiuxian-qianji`.

use qianji_bpmn_engine::{
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ScriptTaskOutcome, ScriptTaskRequest,
    SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome,
    UserTaskRequest,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::defaults::{
    default_clock_handler, unsupported_business_rule_handler, unsupported_event_poll_handler,
    unsupported_manual_handler, unsupported_script_handler, unsupported_send_handler,
    unsupported_service_handler, unsupported_user_handler,
};

pub(super) type HostFuture<T> =
    Pin<Box<dyn Future<Output = std::result::Result<T, HostBridgeError>> + Send + 'static>>;
pub(super) type SendHandler =
    Arc<dyn Fn(SendTaskRequest) -> HostFuture<SendTaskOutcome> + Send + Sync>;
pub(super) type ServiceHandler =
    Arc<dyn Fn(ServiceTaskRequest) -> HostFuture<ServiceTaskOutcome> + Send + Sync>;
pub(super) type ScriptHandler =
    Arc<dyn Fn(ScriptTaskRequest) -> HostFuture<ScriptTaskOutcome> + Send + Sync>;
pub(super) type UserHandler =
    Arc<dyn Fn(UserTaskRequest) -> HostFuture<UserTaskOutcome> + Send + Sync>;
pub(super) type ManualHandler =
    Arc<dyn Fn(ManualTaskRequest) -> HostFuture<ManualTaskOutcome> + Send + Sync>;
pub(super) type BusinessRuleHandler =
    Arc<dyn Fn(BusinessRuleTaskRequest) -> HostFuture<BusinessRuleTaskOutcome> + Send + Sync>;
pub(super) type EventPollHandler =
    Arc<dyn Fn(EventPollRequest) -> HostFuture<EventPollOutcome> + Send + Sync>;
pub(super) type ClockHandler = Arc<dyn Fn() -> u64 + Send + Sync>;

/// Callback-backed `BpmnHostBridge` implementation owned by `xiuxian-qianji`.
#[derive(Clone)]
pub struct QianjiBpmnHostBridge {
    pub(super) send_task: SendHandler,
    pub(super) service_task: ServiceHandler,
    pub(super) script_task: ScriptHandler,
    pub(super) user_task: UserHandler,
    pub(super) manual_task: ManualHandler,
    pub(super) business_rule_task: BusinessRuleHandler,
    pub(super) event_poll: EventPollHandler,
    pub(super) now_unix_ms: ClockHandler,
}

impl QianjiBpmnHostBridge {
    /// Creates a builder for one callback-backed BPMN host bridge.
    #[must_use]
    pub fn builder() -> QianjiBpmnHostBridgeBuilder {
        QianjiBpmnHostBridgeBuilder::default()
    }
}

impl Default for QianjiBpmnHostBridge {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Builder for [`QianjiBpmnHostBridge`].
#[derive(Clone, Default)]
pub struct QianjiBpmnHostBridgeBuilder {
    send_task: Option<SendHandler>,
    service_task: Option<ServiceHandler>,
    script_task: Option<ScriptHandler>,
    user_task: Option<UserHandler>,
    manual_task: Option<ManualHandler>,
    business_rule_task: Option<BusinessRuleHandler>,
    event_poll: Option<EventPollHandler>,
    now_unix_ms: Option<ClockHandler>,
}

impl QianjiBpmnHostBridgeBuilder {
    /// Installs the callback used for BPMN send-task dispatch.
    #[must_use]
    pub fn on_send_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(SendTaskRequest) -> Fut + Send + Sync + 'static,
        Fut:
            Future<Output = std::result::Result<SendTaskOutcome, HostBridgeError>> + Send + 'static,
    {
        self.send_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for BPMN service-task dispatch.
    #[must_use]
    pub fn on_service_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(ServiceTaskRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<ServiceTaskOutcome, HostBridgeError>>
            + Send
            + 'static,
    {
        self.service_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for BPMN script-task dispatch.
    #[must_use]
    pub fn on_script_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(ScriptTaskRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<ScriptTaskOutcome, HostBridgeError>>
            + Send
            + 'static,
    {
        self.script_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for BPMN user-task dispatch.
    #[must_use]
    pub fn on_user_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(UserTaskRequest) -> Fut + Send + Sync + 'static,
        Fut:
            Future<Output = std::result::Result<UserTaskOutcome, HostBridgeError>> + Send + 'static,
    {
        self.user_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for BPMN manual-task dispatch.
    #[must_use]
    pub fn on_manual_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(ManualTaskRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<ManualTaskOutcome, HostBridgeError>>
            + Send
            + 'static,
    {
        self.manual_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for BPMN business-rule dispatch.
    #[must_use]
    pub fn on_business_rule_task<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(BusinessRuleTaskRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<BusinessRuleTaskOutcome, HostBridgeError>>
            + Send
            + 'static,
    {
        self.business_rule_task = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the callback used for external-event polling.
    #[must_use]
    pub fn on_event_poll<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(EventPollRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<EventPollOutcome, HostBridgeError>>
            + Send
            + 'static,
    {
        self.event_poll = Some(Arc::new(move |request| Box::pin(handler(request))));
        self
    }

    /// Installs the wall-clock callback used by the bridge.
    #[must_use]
    pub fn clock<F>(mut self, clock: F) -> Self
    where
        F: Fn() -> u64 + Send + Sync + 'static,
    {
        self.now_unix_ms = Some(Arc::new(clock));
        self
    }

    /// Builds one callback-backed bridge, defaulting unsupported handlers to
    /// explicit `UnsupportedOperation` errors.
    #[must_use]
    pub fn build(self) -> QianjiBpmnHostBridge {
        QianjiBpmnHostBridge {
            send_task: self
                .send_task
                .unwrap_or_else(|| unsupported_send_handler("dispatch_send_task")),
            service_task: self
                .service_task
                .unwrap_or_else(|| unsupported_service_handler("dispatch_service_task")),
            script_task: self
                .script_task
                .unwrap_or_else(|| unsupported_script_handler("dispatch_script_task")),
            user_task: self
                .user_task
                .unwrap_or_else(|| unsupported_user_handler("dispatch_user_task")),
            manual_task: self
                .manual_task
                .unwrap_or_else(|| unsupported_manual_handler("dispatch_manual_task")),
            business_rule_task: self.business_rule_task.unwrap_or_else(|| {
                unsupported_business_rule_handler("dispatch_business_rule_task")
            }),
            event_poll: self
                .event_poll
                .unwrap_or_else(|| unsupported_event_poll_handler("poll_external_event")),
            now_unix_ms: self.now_unix_ms.unwrap_or_else(default_clock_handler),
        }
    }
}
