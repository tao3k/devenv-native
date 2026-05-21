//! `skill_runtime::index` owns Wendao skill runtime index behavior.

mod build;
mod preload;
mod semantic;
mod types;

pub use types::{SkillInventory, SkillInventoryMount, SkillNamespaceIndex, SkillNamespaceMount};
