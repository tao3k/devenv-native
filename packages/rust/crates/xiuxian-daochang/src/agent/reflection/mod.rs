//! Agent reflection branch for lifecycle, policy hints, and turn snapshots.

mod lifecycle;
mod policy_hint;
mod turn;

pub(crate) use lifecycle::{ReflectiveRuntime, ReflectiveRuntimeError, ReflectiveRuntimeStage};
pub(crate) use policy_hint::{PolicyHintDirective, derive_policy_hint};
pub(crate) use turn::{TurnReflection, build_turn_reflection};
