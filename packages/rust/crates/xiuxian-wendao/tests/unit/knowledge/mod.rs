//! Tests for xiuxian-wendao crate.

use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry, KnowledgeSearchQuery, KnowledgeStats};

mod knowledge_category_equality;
/// Test `KnowledgeCategory` enum variants.
mod knowledge_category_variants;
mod knowledge_entry_clone;
mod knowledge_entry_creation;
mod knowledge_entry_default_category;
mod knowledge_entry_equality;
mod knowledge_entry_tag_operations;
mod knowledge_entry_with_metadata;
mod knowledge_entry_with_options;
mod knowledge_stats_default;
mod search_query_builder;
mod search_query_creation;
mod search_query_default;
