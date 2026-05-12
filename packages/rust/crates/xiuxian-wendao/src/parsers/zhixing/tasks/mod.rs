//! `parsers::zhixing::tasks` owns Wendao parsers zhixing tasks behavior.

mod api;
mod identity;
mod metadata;
mod types;

pub use self::api::parse_task_projection;
pub use self::identity::normalize_identity_token;
pub use self::types::TaskLineProjection;

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/zhixing/tasks.rs"]
mod tests;
