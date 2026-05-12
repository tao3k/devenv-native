//! Query and statistics operations for `KnowledgeStorage`.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::types::{KnowledgeEntry, KnowledgeStats};

use super::KnowledgeStorage;
use super::core::saturating_usize_to_i64;
use super::crud::StorageResult;

impl KnowledgeStorage {
    /// Search knowledge entries by vector similarity.
    ///
    /// # Errors
    ///
    /// Returns an error when entry loading fails.
    pub fn search(&self, query: &[f32], limit: i32) -> StorageResult<Vec<KnowledgeEntry>> {
        let take_n = usize::try_from(limit).unwrap_or(0);
        if take_n == 0 {
            return Ok(Vec::new());
        }

        let query_norm = self.normalize_vector(query);
        let mut scored: Vec<(f32, KnowledgeEntry)> = self
            .load_all_entries()?
            .into_iter()
            .map(|entry| {
                let score =
                    Self::cosine_similarity(&query_norm, &self.text_to_vector(&entry.content));
                (score, entry)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        Ok(scored
            .into_iter()
            .take(take_n)
            .map(|(_, entry)| entry)
            .collect())
    }

    /// Search knowledge entries by text.
    ///
    /// # Errors
    ///
    /// Returns an error when entry loading fails.
    pub fn search_text(
        &self,
        query: impl AsRef<str>,
        limit: i32,
    ) -> StorageResult<Vec<KnowledgeEntry>> {
        let query = query.as_ref();
        let take_n = usize::try_from(limit).unwrap_or(0);
        if take_n == 0 {
            return Ok(Vec::new());
        }

        let mut scored: Vec<(f32, KnowledgeEntry)> = self
            .load_all_entries()?
            .into_iter()
            .map(|entry| {
                let mut relevance_score = Self::text_score(query, &entry);
                if entry.id == query {
                    relevance_score += 1.0;
                }
                let category_bonus = if query
                    .to_ascii_lowercase()
                    .contains(Self::category_to_str(entry.category))
                {
                    0.1
                } else {
                    0.0
                };
                (relevance_score + category_bonus, entry)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        Ok(scored.into_iter().take(take_n).map(|(_, e)| e).collect())
    }

    /// Get statistics about the knowledge base.
    ///
    /// # Errors
    ///
    /// Returns an error when entry loading fails.
    pub fn stats(&self) -> StorageResult<KnowledgeStats> {
        let entries = self.load_all_entries()?;
        if entries.is_empty() {
            return Ok(KnowledgeStats::default());
        }

        let mut by_category: HashMap<String, i64> = HashMap::new();
        let mut unique_tags: HashSet<String> = HashSet::new();
        let mut last_updated = entries[0].updated_at;

        for entry in &entries {
            let key = serde_json::to_string(&entry.category)
                .unwrap_or_else(|_| "\"notes\"".to_string())
                .trim_matches('"')
                .to_string();
            *by_category.entry(key).or_insert(0) += 1;

            for tag in &entry.tags {
                unique_tags.insert(tag.to_lowercase());
            }
            if entry.updated_at > last_updated {
                last_updated = entry.updated_at;
            }
        }

        Ok(KnowledgeStats {
            total_entries: saturating_usize_to_i64(entries.len()),
            entries_by_category: by_category,
            total_tags: saturating_usize_to_i64(unique_tags.len()),
            last_updated: Some(last_updated),
        })
    }
}
