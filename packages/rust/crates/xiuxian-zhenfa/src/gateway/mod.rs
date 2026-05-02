//! HTTP gateway assembly, handlers, and webhook notification support.

mod builder;
mod handlers;
mod notification;

pub use builder::{ZhenfaGatewayBuildError, ZhenfaGatewayBuilder};
pub use handlers::HealthResponse;
pub use notification::{
    NotificationError, NotificationPayload, NotificationService, WebhookConfig, notification_worker,
};
