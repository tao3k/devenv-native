//! `qianji control` command surface.

mod activity_admit_plan;
mod activity_args;
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
mod activity_artifact;
mod activity_claim;
mod activity_executor;
mod activity_finish;
mod activity_mirror;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
mod activity_openai_compatible;
mod activity_reclaim;
mod activity_release;
mod activity_schedule_llm;
mod activity_settle;
mod activity_start;
mod activity_take;
mod activity_worker_loop;
mod activity_worker_once;
mod api;
mod heartbeat;
mod llm_inventory;
mod parse;
mod render;
mod run;
mod run_create;
mod types;

#[cfg(test)]
pub(crate) use activity_args::ActivitySettleOutcomeArg;
#[cfg(test)]
pub(crate) use activity_claim::{WorkerActivityClaimStoreRequest, claim_with_hot_state};
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) use activity_executor::ActivityExecutorOutcome;
#[cfg(test)]
pub(crate) use activity_executor::{
    ActivityExecutionRequest, ActivityExecutorAdapterKind, ActivityExecutorKindArg,
    ActivityExecutorRegistry,
};
#[cfg(test)]
pub(crate) use activity_mirror::{WorkerActivityMirrorStoreRequest, mirror_with_hot_state};
#[cfg(test)]
pub(crate) use activity_reclaim::{WorkerActivityReclaimStoreRequest, reclaim_with_hot_state};
#[cfg(test)]
pub(crate) use activity_release::{WorkerActivityReleaseStoreRequest, release_with_hot_state};
#[cfg(test)]
pub(crate) use activity_settle::{WorkerActivitySettleStoreRequest, settle_with_hot_state};
#[cfg(test)]
pub(crate) use activity_take::{WorkerActivityTakeStoreRequest, take_with_hot_state};
#[cfg(test)]
pub(crate) use activity_worker_loop::{ActivityWorkerLoopStoreRequest, worker_loop_with_hot_state};
#[cfg(test)]
pub(crate) use activity_worker_once::{ActivityWorkerOnceStoreRequest, worker_once_with_hot_state};
#[cfg(test)]
pub(crate) use api::run_control_command;
pub(crate) use api::{handle_control_command_async, parse_control_command};
#[cfg(test)]
pub(crate) use heartbeat::{HeartbeatHotStateRequest, heartbeat_with_hot_state};
pub(crate) use types::{ControlCliCommand, ControlCliOutput};
