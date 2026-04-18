//! Cargo entry point for `xiuxian-skills` unit tests.

xiuxian_testing::crate_test_policy_harness!();

#[path = "unit/frontmatter.rs"]
mod frontmatter;
#[path = "unit/skills/metadata/reference_record.rs"]
mod metadata_reference_record;
#[path = "unit/skills/metadata/sync.rs"]
mod metadata_sync;
#[path = "unit/skills/metadata/tool_record.rs"]
mod metadata_tool_record;
#[path = "unit/skills/prompt.rs"]
mod prompt;
#[path = "unit/schema_generation.rs"]
mod schema_generation;
#[path = "unit/schema_validation.rs"]
mod schema_validation;
#[path = "unit/skills/skill_command/parser.rs"]
mod skill_command_parser;
#[path = "unit/skills/skill_command/parser_parameters_model.rs"]
mod skill_command_parser_parameters_model;
#[path = "unit/skill_metadata.rs"]
mod skill_metadata;
#[path = "unit/skill_scanner_behavior.rs"]
mod skill_scanner_behavior;
#[path = "unit/skill_scanner_structure/mod.rs"]
mod skill_scanner_structure;
#[path = "unit/skill_structure_config_cascade.rs"]
mod skill_structure_config_cascade;
#[path = "unit/skills/tools/mod.rs"]
mod skills_tools;
#[path = "unit/tools_scanner/mod.rs"]
mod tools_scanner;
