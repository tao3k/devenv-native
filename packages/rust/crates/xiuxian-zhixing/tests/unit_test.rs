//! Canonical unit test harness for `xiuxian-zhixing`.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/test_agenda_entry.rs"]
mod test_agenda_entry;
#[path = "unit/test_forge_skill_resources.rs"]
mod test_forge_skill_resources;
#[path = "unit/test_heyi/mod.rs"]
mod test_heyi;
#[path = "unit/test_reminder_queue.rs"]
mod test_reminder_queue;
#[path = "unit/test_storage_markdown.rs"]
mod test_storage_markdown;
#[path = "unit/test_strict_teacher.rs"]
mod test_strict_teacher;
#[path = "unit/test_units.rs"]
mod test_units;
#[path = "unit/test_wendao_indexer.rs"]
mod test_wendao_indexer;
#[path = "unit/test_wendao_skill_resources.rs"]
mod test_wendao_skill_resources;
