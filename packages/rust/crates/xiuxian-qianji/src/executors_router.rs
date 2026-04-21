//! Probabilistic MDP routing mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use rand::Rng;
use serde_json::json;

/// Mechanism responsible for dynamic probabilistic path selection.
pub struct ProbabilisticRouter {
    /// List of available branches and their relative weights.
    pub branches: Vec<(String, f32)>, // (BranchName, StaticWeight)
}

#[async_trait]
impl QianjiMechanism for ProbabilisticRouter {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        if self.branches.is_empty() {
            return Err("Router has no branches configured".to_string());
        }

        let confidence_bias = confidence_bias(context)?;
        let mut eligible: Vec<(&String, f32)> = Vec::new();
        for (name, weight) in &self.branches {
            let scaled = *weight * confidence_bias;
            if !scaled.is_finite() {
                return Err("Router branch weight produced a non-finite score".to_string());
            }
            if scaled > 0.0 {
                eligible.push((name, scaled));
            }
        }
        if eligible.is_empty() {
            return Err("Router has no positive branch weights".to_string());
        }

        let total_weight: f32 = eligible.iter().map(|(_, w)| *w).sum();
        let mut rng = rand::thread_rng();
        let mut pick = rng.gen_range(0.0..total_weight);
        let mut selected_branch = eligible[0].0.clone();
        for (name, weight) in eligible {
            pick -= weight;
            if pick <= 0.0 {
                selected_branch.clone_from(name);
                break;
            }
        }

        Ok(QianjiOutput {
            data: json!({ "selected_route": selected_branch }),
            instruction: FlowInstruction::SelectBranch(selected_branch),
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn confidence_bias(context: &serde_json::Value) -> Result<f32, String> {
    let raw = context
        .get("omega_confidence")
        .map_or(Ok(1.0_f32), |value| {
            serde_json::from_value::<f32>(value.clone())
                .map_err(|_error| "omega_confidence must be a finite number".to_string())
        })?;
    let bias = validate_f32(raw, "omega_confidence")?;
    if bias <= 0.0 {
        return Err("omega_confidence must be positive".to_string());
    }
    Ok(bias)
}

fn validate_f32(value: f32, field: &str) -> Result<f32, String> {
    if !value.is_finite() {
        return Err(format!("{field} must be finite"));
    }
    Ok(value)
}

#[cfg(test)]
#[path = "../tests/unit/executors/router.rs"]
mod tests;
