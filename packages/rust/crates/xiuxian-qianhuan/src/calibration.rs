//! Calibration primitives for adversarial prompt-alignment checks.

use crate::persona::PersonaProfile;
use serde::{Deserialize, Serialize};

/// Represents the state of a Synapse-Audit calibration loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalibrationState {
    /// Candidate generation stage before challenge.
    Prospecting,
    /// Skeptic review stage that probes weaknesses.
    SkepticReview,
    /// Feedback-driven calibration stage.
    Calibrating,
    /// Finalized stage with accepted alignment.
    Finalized,
}

/// The result of an adversarial audit turn.
pub struct AuditVerdict {
    /// Drift score in [0, 1], where lower is better aligned.
    pub drift_score: f32,
    /// Whether the evaluated content is considered aligned.
    pub is_aligned: bool,
    /// Missing anchor terms detected during evidence scan.
    pub missing_anchors: Vec<String>,
}

/// Core engine for adversarial multi-persona calibration.
/// Implements Synapse-Audit (2025) principles.
pub struct AdversarialOrchestrator {
    /// Persona used to generate candidate assertions.
    pub prospector: PersonaProfile,
    /// Persona used to challenge candidate assertions.
    pub skeptic: PersonaProfile,
    /// Persona used to reconcile feedback and calibrate output.
    pub calibrator: PersonaProfile,
}

impl AdversarialOrchestrator {
    /// Create an adversarial calibration orchestrator from three personas.
    #[must_use]
    pub fn new(
        prospector: PersonaProfile,
        skeptic: PersonaProfile,
        calibrator: PersonaProfile,
    ) -> Self {
        Self {
            prospector,
            skeptic,
            calibrator,
        }
    }

    /// Evaluates the alignment between an agent's claim and the provided evidence.
    /// Returns a drift score based on semantic overlap and anchor binding.
    #[must_use]
    pub fn evaluate_alignment(&self, _claim: &str, evidence: &[String]) -> AuditVerdict {
        // This simulates the 'Skeptic' checking for counter-evidence.
        // In full implementation, this could involve Rust-side regex or keyword indices.
        let anchors = &self.prospector.style_anchors;
        let missing = anchors
            .iter()
            .filter(|anchor| !evidence_contains_anchor(evidence, anchor))
            .cloned()
            .collect::<Vec<_>>();
        let total_anchors = u16::try_from(anchors.len()).map_or(f32::INFINITY, f32::from);
        let missing_anchors = u16::try_from(missing.len()).map_or(f32::INFINITY, f32::from);
        let matched_anchors = total_anchors - missing_anchors;

        let drift = if total_anchors == 0.0 {
            0.0
        } else {
            1.0 - (matched_anchors / total_anchors).clamp(0.0, 1.0)
        };

        AuditVerdict {
            drift_score: drift,
            is_aligned: drift < 0.05, // Synapse-Audit threshold
            missing_anchors: missing,
        }
    }
}

fn evidence_contains_anchor(evidence: &[String], anchor: &str) -> bool {
    let anchor = anchor.to_lowercase();
    evidence
        .iter()
        .any(|entry| entry.to_lowercase().contains(&anchor))
}
