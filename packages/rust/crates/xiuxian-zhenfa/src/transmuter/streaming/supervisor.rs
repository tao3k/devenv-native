//! Cognitive Supervisor: Thought Dimension Analysis (V3.0).
//!
//! This module categorizes streaming events into cognitive dimensions,
//! distinguishing between Meta-level (planning, self-reflection),
//! Operational-level (task execution, implementation), and
//! Epistemic-level (uncertainty, knowledge gaps) thoughts.
//!
//! ## V3.0 Features
//!
//! - **Three-Dimensional Cognitive Model**: `Meta`, `Operational`, `Epistemic`
//! - **Coherence Score**: Real-time quality assessment for Early-Halt
//! - **Hallucination Defense**: Second line of defense after `LogicGate` XSD validation
//! - **`VecDeque` History**: O(1) front removal for bounded history

use super::ZhenfaStreamingEvent;
use std::collections::VecDeque;

/// Maximum history size for cognitive tracking.
pub(crate) const MAX_HISTORY_SIZE: usize = 100;

/// Cognitive dimension classification for agent thoughts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveDimension {
    /// Meta-level cognition: planning, self-reflection, strategy evaluation.
    Meta,
    /// Operational-level cognition: task execution, implementation reasoning.
    Operational,
    /// Epistemic-level cognition: uncertainty, knowledge gaps, confidence assessment.
    Epistemic,
    /// System-level events: status, progress, errors.
    System,
    /// Tool interaction: calls and results.
    Instrumental,
}

/// Analyzed cognitive event with dimension classification.
#[derive(Debug, Clone, PartialEq)]
pub struct CognitiveEvent {
    /// The original streaming event.
    pub source: ZhenfaStreamingEvent,
    /// Classified cognitive dimension.
    pub dimension: CognitiveDimension,
    /// Optional sub-category for finer granularity.
    pub subcategory: Option<ThoughtSubcategory>,
    /// Confidence score for the classification (0.0-1.0).
    pub confidence: f32,
    /// Coherence score for this event (V2.0).
    pub coherence: f32,
}

/// Fine-grained subcategory for thought events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThoughtSubcategory {
    // Meta subcategories
    /// Planning future actions.
    Planning,
    /// Self-reflection on current approach.
    SelfReflection,
    /// Evaluating alternative strategies.
    StrategyEvaluation,
    /// Analyzing past errors.
    ErrorAnalysis,
    /// Setting or adjusting goals.
    GoalSetting,

    // Operational subcategories
    /// Analyzing code or file structure.
    CodeAnalysis,
    /// Reasoning about implementation details.
    ImplementationReasoning,
    /// Debugging active problems.
    Debugging,
    /// Reviewing or validating changes.
    Validation,
    /// Searching or exploring codebase.
    Exploration,

    // Epistemic subcategories (V2.0)
    /// Expressing uncertainty about approach.
    Uncertainty,
    /// Identifying knowledge gaps.
    KnowledgeGap,
    /// Seeking clarification or additional context.
    ClarificationSeeking,
    /// Assessing confidence in current solution.
    ConfidenceAssessment,
}

type ThoughtPattern = (&'static str, ThoughtSubcategory);

const META_PATTERNS: &[ThoughtPattern] = &[
    ("i should", ThoughtSubcategory::Planning),
    ("let me plan", ThoughtSubcategory::Planning),
    ("first, i'll", ThoughtSubcategory::Planning),
    ("my approach", ThoughtSubcategory::Planning),
    ("the plan is", ThoughtSubcategory::Planning),
    (
        "wait, let me reconsider",
        ThoughtSubcategory::SelfReflection,
    ),
    ("actually, i think", ThoughtSubcategory::SelfReflection),
    ("on second thought", ThoughtSubcategory::SelfReflection),
    ("i realize that", ThoughtSubcategory::SelfReflection),
    ("let me think about", ThoughtSubcategory::SelfReflection),
    (
        "alternative approach",
        ThoughtSubcategory::StrategyEvaluation,
    ),
    ("another way", ThoughtSubcategory::StrategyEvaluation),
    ("better approach", ThoughtSubcategory::StrategyEvaluation),
    ("instead of", ThoughtSubcategory::StrategyEvaluation),
    ("the error", ThoughtSubcategory::ErrorAnalysis),
    ("went wrong", ThoughtSubcategory::ErrorAnalysis),
    ("failed because", ThoughtSubcategory::ErrorAnalysis),
    ("the issue is", ThoughtSubcategory::ErrorAnalysis),
    ("my goal", ThoughtSubcategory::GoalSetting),
    ("the objective", ThoughtSubcategory::GoalSetting),
    ("what i want to achieve", ThoughtSubcategory::GoalSetting),
];

const OPERATIONAL_PATTERNS: &[ThoughtPattern] = &[
    ("this function", ThoughtSubcategory::CodeAnalysis),
    ("the code", ThoughtSubcategory::CodeAnalysis),
    ("this file", ThoughtSubcategory::CodeAnalysis),
    ("the implementation", ThoughtSubcategory::CodeAnalysis),
    ("this module", ThoughtSubcategory::CodeAnalysis),
    ("i'll add", ThoughtSubcategory::ImplementationReasoning),
    ("i'll modify", ThoughtSubcategory::ImplementationReasoning),
    (
        "i need to change",
        ThoughtSubcategory::ImplementationReasoning,
    ),
    ("the fix is", ThoughtSubcategory::ImplementationReasoning),
    ("debugging", ThoughtSubcategory::Debugging),
    ("trace the issue", ThoughtSubcategory::Debugging),
    ("the bug", ThoughtSubcategory::Debugging),
    ("verify that", ThoughtSubcategory::Validation),
    ("check if", ThoughtSubcategory::Validation),
    ("ensure that", ThoughtSubcategory::Validation),
    ("confirm that", ThoughtSubcategory::Validation),
    ("let me search", ThoughtSubcategory::Exploration),
    ("looking for", ThoughtSubcategory::Exploration),
    ("i need to find", ThoughtSubcategory::Exploration),
    ("exploring", ThoughtSubcategory::Exploration),
];

const EPISTEMIC_PATTERNS: &[ThoughtPattern] = &[
    ("i'm not sure", ThoughtSubcategory::Uncertainty),
    ("i'm uncertain", ThoughtSubcategory::Uncertainty),
    ("i don't know", ThoughtSubcategory::Uncertainty),
    ("unclear", ThoughtSubcategory::Uncertainty),
    ("might be", ThoughtSubcategory::Uncertainty),
    ("i need more context", ThoughtSubcategory::KnowledgeGap),
    ("i need to understand", ThoughtSubcategory::KnowledgeGap),
    ("not familiar with", ThoughtSubcategory::KnowledgeGap),
    ("i haven't seen", ThoughtSubcategory::KnowledgeGap),
    ("let me clarify", ThoughtSubcategory::ClarificationSeeking),
    ("can you clarify", ThoughtSubcategory::ClarificationSeeking),
    ("i should ask", ThoughtSubcategory::ClarificationSeeking),
    ("i'm confident", ThoughtSubcategory::ConfidenceAssessment),
    ("my confidence", ThoughtSubcategory::ConfidenceAssessment),
    ("fairly certain", ThoughtSubcategory::ConfidenceAssessment),
    ("pretty sure", ThoughtSubcategory::ConfidenceAssessment),
];

/// Coherence metrics for Early-Halt decision making.
#[derive(Debug, Clone, Default)]
pub struct CoherenceMetrics {
    /// Running coherence score (0.0-1.0).
    pub score: f32,
    /// Number of incoherent events detected.
    pub incoherent_count: u32,
    /// Number of self-correction events.
    pub self_correction_count: u32,
    /// Whether Early-Halt threshold has been breached.
    pub early_halt_triggered: bool,
}

/// Pattern matcher for cognitive dimension detection.
#[derive(Debug, Default)]
pub struct CognitiveSupervisor {
    /// History of classified thoughts for context (`VecDeque` for O(1) front removal).
    thought_history: VecDeque<CognitiveDimension>,
    /// Current operational context.
    context: SupervisorContext,
    /// Coherence metrics for Early-Halt (V2.0).
    coherence: CoherenceMetrics,
    /// Threshold for triggering Early-Halt.
    early_halt_threshold: f32,
}

/// Operational context for more accurate classification.
#[derive(Debug, Clone, Default)]
pub struct SupervisorContext {
    /// Whether the agent is in an active implementation phase.
    pub in_implementation: bool,
    /// Whether the agent has encountered recent errors.
    pub has_recent_errors: bool,
    /// Number of consecutive meta thoughts.
    pub meta_streak: u32,
    /// Number of consecutive operational thoughts.
    pub operational_streak: u32,
    /// Number of consecutive epistemic thoughts (V2.0).
    pub epistemic_streak: u32,
    /// Number of consecutive uncertainty expressions (V2.0).
    pub uncertainty_streak: u32,
}

fn saturating_u32_to_f32(value: u32) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

fn saturating_usize_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

fn ratio_from_counts(numerator: usize, denominator: usize) -> f32 {
    debug_assert!(denominator > 0);
    saturating_usize_to_f32(numerator) / saturating_usize_to_f32(denominator)
}

fn pattern_confidence(pattern: &str, text_len: usize, base: f32, bonus_cap: f32) -> f32 {
    base + (ratio_from_counts(pattern.len(), text_len.max(1))).min(bonus_cap)
}

fn best_pattern_match(
    text_lower: &str,
    text_len: usize,
    patterns: &[ThoughtPattern],
    base: f32,
    bonus_cap: f32,
) -> Option<(ThoughtSubcategory, f32)> {
    let mut best_match = None;
    for (pattern, subcategory) in patterns {
        if text_lower.contains(pattern) {
            let confidence = pattern_confidence(pattern, text_len, base, bonus_cap);
            match best_match {
                None => best_match = Some((*subcategory, confidence)),
                Some((_, existing_confidence)) if confidence > existing_confidence => {
                    best_match = Some((*subcategory, confidence));
                }
                _ => {}
            }
        }
    }
    best_match
}

impl CognitiveSupervisor {
    /// Create a new cognitive supervisor with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a cognitive supervisor with custom Early-Halt threshold.
    #[must_use]
    pub fn with_early_halt_threshold(threshold: f32) -> Self {
        Self {
            early_halt_threshold: threshold.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Classify a streaming event into its cognitive dimension.
    #[must_use]
    pub fn classify(&mut self, event: ZhenfaStreamingEvent) -> CognitiveEvent {
        let (dimension, subcategory, confidence) = self.analyze_event(&event);

        // Track error state
        if matches!(event, ZhenfaStreamingEvent::Error { .. }) {
            self.context.has_recent_errors = true;
        }

        // Update context based on classification
        self.update_context(dimension);

        // Calculate coherence score (V2.0)
        let coherence = self.calculate_coherence(dimension, subcategory);

        CognitiveEvent {
            source: event,
            dimension,
            subcategory,
            confidence,
            coherence,
        }
    }

    /// Get current coherence metrics.
    #[must_use]
    pub const fn coherence(&self) -> &CoherenceMetrics {
        &self.coherence
    }

    /// Check if Early-Halt should be triggered.
    #[must_use]
    pub const fn should_halt(&self) -> bool {
        self.coherence.early_halt_triggered
    }

    /// Get the configured Early-Halt threshold.
    #[must_use]
    pub const fn early_halt_threshold(&self) -> f32 {
        self.early_halt_threshold
    }

    /// Calculate coherence score for an event.
    fn calculate_coherence(
        &mut self,
        dimension: CognitiveDimension,
        subcategory: Option<ThoughtSubcategory>,
    ) -> f32 {
        // Base coherence
        let mut coherence = 0.8;

        // Reduce coherence for epistemic uncertainty
        if dimension == CognitiveDimension::Epistemic {
            coherence -= 0.15;
            self.context.uncertainty_streak += 1;

            // Excessive uncertainty streak reduces coherence further
            if self.context.uncertainty_streak > 3 {
                coherence -= 0.1 * saturating_u32_to_f32(self.context.uncertainty_streak - 3);
            }
        } else {
            self.context.uncertainty_streak = 0;
        }

        // Self-reflection indicates coherence (meta-awareness)
        if matches!(subcategory, Some(ThoughtSubcategory::SelfReflection)) {
            coherence += 0.1;
            self.coherence.self_correction_count += 1;
        }

        // Error context reduces coherence
        if self.context.has_recent_errors {
            coherence -= 0.1;
        }

        // Long meta streak without operational action
        if self.context.meta_streak > 5 {
            coherence -= 0.05 * saturating_u32_to_f32(self.context.meta_streak - 5);
        }

        // Detect incoherent patterns (oscillating between meta/operational without progress)
        if self.is_oscillating() {
            coherence -= 0.15;
            self.coherence.incoherent_count += 1;
        }

        coherence = coherence.clamp(0.0, 1.0);
        self.coherence.score = coherence;

        // Check Early-Halt threshold
        if coherence < self.early_halt_threshold && self.early_halt_threshold > 0.0 {
            self.coherence.early_halt_triggered = true;
        }

        coherence
    }

    /// Detect oscillating behavior (rapid switching between dimensions).
    fn is_oscillating(&self) -> bool {
        if self.thought_history.len() < 4 {
            return false;
        }

        // Check last 4 entries for pattern: A-B-A-B using VecDeque iterators
        let last_4: Vec<_> = self.thought_history.iter().rev().take(4).collect();

        // Pattern: Meta-Op-Meta-Op or Op-Meta-Op-Meta (reversed order from rev())
        matches!(
            (last_4[0], last_4[1], last_4[2], last_4[3]),
            (
                CognitiveDimension::Operational,
                CognitiveDimension::Meta,
                CognitiveDimension::Operational,
                CognitiveDimension::Meta
            ) | (
                CognitiveDimension::Meta,
                CognitiveDimension::Operational,
                CognitiveDimension::Meta,
                CognitiveDimension::Operational
            )
        )
    }

    /// Analyze an event and return (dimension, subcategory, confidence).
    fn analyze_event(
        &self,
        event: &ZhenfaStreamingEvent,
    ) -> (CognitiveDimension, Option<ThoughtSubcategory>, f32) {
        match event {
            ZhenfaStreamingEvent::Thought(text) => self.classify_thought(text),
            ZhenfaStreamingEvent::TextDelta(_) => (CognitiveDimension::Operational, None, 0.9),
            ZhenfaStreamingEvent::ToolCall { .. } => (CognitiveDimension::Instrumental, None, 1.0),
            ZhenfaStreamingEvent::ToolResult { .. } => {
                (CognitiveDimension::Instrumental, None, 1.0)
            }
            ZhenfaStreamingEvent::Status(_)
            | ZhenfaStreamingEvent::Progress { .. }
            | ZhenfaStreamingEvent::Finished(_)
            | ZhenfaStreamingEvent::Error { .. } => (CognitiveDimension::System, None, 1.0),
        }
    }

    /// Classify a thought text into cognitive dimension.
    fn classify_thought(
        &self,
        text: &str,
    ) -> (CognitiveDimension, Option<ThoughtSubcategory>, f32) {
        let text_lower = text.to_lowercase();
        let text_len = text.len();
        let best_epistemic =
            best_pattern_match(&text_lower, text_len, EPISTEMIC_PATTERNS, 0.75, 0.2);

        // If epistemic pattern matched strongly, return it
        if let Some((subcategory, confidence)) = best_epistemic {
            return (CognitiveDimension::Epistemic, Some(subcategory), confidence);
        }

        let best_meta = best_pattern_match(&text_lower, text_len, META_PATTERNS, 0.7, 0.25);
        let best_operational =
            best_pattern_match(&text_lower, text_len, OPERATIONAL_PATTERNS, 0.7, 0.25);

        // Decide between meta and operational
        match (best_meta, best_operational) {
            (Some((meta_sub, meta_conf)), Some((op_sub, op_conf))) => {
                // Both matched - use context to break tie
                if meta_conf > op_conf {
                    (CognitiveDimension::Meta, Some(meta_sub), meta_conf)
                } else if op_conf > meta_conf {
                    (CognitiveDimension::Operational, Some(op_sub), op_conf)
                } else {
                    // Equal confidence - use context
                    if self.context.in_implementation {
                        (CognitiveDimension::Operational, Some(op_sub), op_conf)
                    } else {
                        (CognitiveDimension::Meta, Some(meta_sub), meta_conf)
                    }
                }
            }
            (Some((subcategory, confidence)), None) => {
                (CognitiveDimension::Meta, Some(subcategory), confidence)
            }
            (None, Some((subcategory, confidence))) => (
                CognitiveDimension::Operational,
                Some(subcategory),
                confidence,
            ),
            (None, None) => Self::classify_by_heuristics(text),
        }
    }

    /// Fallback classification using heuristics when no patterns match.
    fn classify_by_heuristics(text: &str) -> (CognitiveDimension, Option<ThoughtSubcategory>, f32) {
        let text_lower = text.to_lowercase();

        // Check for question marks (often meta-reflection)
        let question_count = text.matches('?').count();

        // Check for first-person pronouns (meta-indicator)
        let first_person_indicators = text_lower.matches(" i ").count()
            + text_lower.matches(" i'm ").count()
            + text_lower.matches(" my ").count();

        // Check for action verbs (operational-indicator)
        let action_verbs = [
            "add",
            "remove",
            "modify",
            "change",
            "create",
            "delete",
            "implement",
            "fix",
            "update",
            "refactor",
            "write",
        ];
        let action_count = action_verbs
            .iter()
            .filter(|verb| text_lower.contains(*verb))
            .count();

        // Heuristic scoring
        let meta_score = saturating_usize_to_f32(question_count) * 0.2
            + saturating_usize_to_f32(first_person_indicators) * 0.3;
        let operational_score = saturating_usize_to_f32(action_count) * 0.25;

        if meta_score > operational_score {
            (CognitiveDimension::Meta, None, 0.5 + meta_score * 0.1)
        } else if operational_score > 0.0 {
            (
                CognitiveDimension::Operational,
                None,
                0.5 + operational_score * 0.1,
            )
        } else {
            // Default to operational with low confidence
            (CognitiveDimension::Operational, None, 0.4)
        }
    }

    /// Update the supervisor context based on classification.
    fn update_context(&mut self, dimension: CognitiveDimension) {
        match dimension {
            CognitiveDimension::Meta => {
                self.context.meta_streak += 1;
                self.context.operational_streak = 0;
                self.context.epistemic_streak = 0;
            }
            CognitiveDimension::Operational => {
                self.context.operational_streak += 1;
                self.context.meta_streak = 0;
                self.context.epistemic_streak = 0;
                self.context.in_implementation = true;
            }
            CognitiveDimension::Epistemic => {
                self.context.epistemic_streak += 1;
                self.context.meta_streak = 0;
                self.context.operational_streak = 0;
            }
            CognitiveDimension::System => {
                // System events don't change the streak
            }
            CognitiveDimension::Instrumental => {
                // Tool interactions indicate operational phase
                self.context.in_implementation = true;
            }
        }

        self.thought_history.push_back(dimension);

        // Keep history bounded with O(1) front removal
        if self.thought_history.len() > MAX_HISTORY_SIZE {
            self.thought_history.pop_front();
        }
    }

    /// Get the current context.
    #[cfg(test)]
    #[must_use]
    pub fn context(&self) -> &SupervisorContext {
        &self.context
    }

    /// Reset the supervisor state.
    pub fn reset(&mut self) {
        self.thought_history.clear();
        self.context = SupervisorContext::default();
        self.coherence = CoherenceMetrics::default();
    }

    /// Get the cognitive balance (ratio of meta to operational thoughts).
    #[cfg(test)]
    #[must_use]
    pub fn cognitive_balance(&self) -> f32 {
        if self.thought_history.is_empty() {
            return 0.5;
        }

        let meta_count = self
            .thought_history
            .iter()
            .filter(|&&d| d == CognitiveDimension::Meta)
            .count();

        ratio_from_counts(meta_count, self.thought_history.len())
    }

    /// Get the current history length.
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.thought_history.len()
    }

    /// Get a slice of the history.
    #[must_use]
    pub fn history_slice(&self, start: usize, end: usize) -> Vec<CognitiveDimension> {
        self.thought_history
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .copied()
            .collect()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/transmuter/streaming/supervisor.rs"]
mod tests;
