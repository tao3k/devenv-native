//! Probabilistic MDP routing mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use rand::Rng;
use serde_json::{Value, json};

/// Mechanism responsible for dynamic probabilistic path selection.
pub struct ProbabilisticRouter {
    /// List of available branches and their relative weights.
    pub branches: Vec<(String, f32)>, // (BranchName, StaticWeight)
    /// Optional context key containing a semantic guard route object.
    pub semantic_guard_route_key: Option<String>,
}

#[async_trait]
impl QianjiMechanism for ProbabilisticRouter {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        if self.branches.is_empty() {
            return Err("Router has no branches configured".to_string());
        }

        if let Some(selected_branch) = semantic_guard_route_branch(
            context,
            self.semantic_guard_route_key.as_deref(),
            &self.branches,
        )? {
            return Ok(router_output(selected_branch));
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

        Ok(router_output(selected_branch))
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn router_output(selected_branch: String) -> QianjiOutput {
    QianjiOutput {
        data: json!({ "selected_route": selected_branch }),
        instruction: FlowInstruction::SelectBranch(selected_branch),
    }
}

fn semantic_guard_route_branch(
    context: &Value,
    route_key: Option<&str>,
    branches: &[(String, f32)],
) -> Result<Option<String>, String> {
    let Some(route_key) = route_key else {
        return Ok(None);
    };
    let Some(route) = context.get(route_key) else {
        return Ok(None);
    };
    if route.is_null() {
        return Ok(None);
    }
    let Some(route) = route.as_object() else {
        return Err(format!(
            "{route_key} must be an object when semantic guard routing is enabled"
        ));
    };
    let Some(action) = route
        .get("recommendedAction")
        .or_else(|| route.get("recommended_action"))
    else {
        return Ok(None);
    };
    let Some(action) = action.as_str() else {
        return Err(format!("{route_key}.recommendedAction must be a string"));
    };
    let action = action.trim();
    if action.is_empty() {
        return Ok(None);
    }
    Ok(branches
        .iter()
        .find(|(name, weight)| name == action && weight.is_finite() && *weight > 0.0)
        .map(|(name, _weight)| name.clone()))
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
