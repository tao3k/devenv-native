//! Canonical unit test harness for `xiuxian-wendao-parsers`.

rust_lang_project_harness::rust_project_harness_gate!();

#[path = "unit/blocks.rs"]
mod blocks;
#[path = "unit/code_observation.rs"]
mod code_observation;
#[path = "unit/document.rs"]
mod document;
#[path = "unit/frontmatter.rs"]
mod frontmatter;
#[path = "unit/lint/mod.rs"]
mod lint;
#[path = "unit/note.rs"]
mod note;
#[path = "unit/org.rs"]
mod org;
#[path = "unit/references.rs"]
mod references;
#[path = "unit/section_create.rs"]
mod section_create;
#[path = "unit/sections.rs"]
mod sections;
#[path = "unit/targets.rs"]
mod targets;
#[path = "unit/toc.rs"]
mod toc;
#[path = "unit/wikilinks.rs"]
mod wikilinks;
