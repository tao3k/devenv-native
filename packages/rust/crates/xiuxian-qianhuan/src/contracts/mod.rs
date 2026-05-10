//! Prompt-injection contract records shared by Qianhuan rendering layers.

mod block;
mod policy;
mod role_mix;
mod snapshot;

pub use block::{
    PromptContextBlock, PromptContextBlockId, PromptContextBlockInput, PromptContextCategory,
    PromptContextSource, PromptSessionScope,
};
pub use policy::{InjectionMode, InjectionOrderStrategy, InjectionPolicy};
pub use role_mix::{RoleMixProfile, RoleMixRole};
pub use snapshot::{
    InjectionSessionId, InjectionSnapshot, InjectionSnapshotId, InjectionSnapshotInput,
    InjectionTurnId,
};
