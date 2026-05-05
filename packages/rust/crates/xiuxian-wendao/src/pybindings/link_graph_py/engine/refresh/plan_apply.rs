use crate::pybindings::link_graph_py::engine::PyLinkGraphEngine;
use pyo3::PyResult;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;

use crate::link_graph::LinkGraphRefreshMode;

pub(super) enum RefreshPlanStrategy {
    Noop,
    Full {
        reason: &'static str,
    },
    Delta {
        reason: &'static str,
        prefer_incremental: bool,
    },
}

pub(super) fn select_refresh_strategy(
    force_full: bool,
    changed_count: usize,
    threshold: usize,
) -> RefreshPlanStrategy {
    if force_full {
        RefreshPlanStrategy::Full {
            reason: "force_full",
        }
    } else if changed_count == 0 {
        RefreshPlanStrategy::Noop
    } else if changed_count >= threshold.max(1) {
        RefreshPlanStrategy::Delta {
            reason: "threshold_exceeded_incremental",
            prefer_incremental: true,
        }
    } else {
        RefreshPlanStrategy::Delta {
            reason: "delta_requested",
            prefer_incremental: false,
        }
    }
}

pub(super) fn strategy_label_and_reason(
    strategy: &RefreshPlanStrategy,
) -> (&'static str, &'static str) {
    match strategy {
        RefreshPlanStrategy::Noop => ("noop", "noop"),
        RefreshPlanStrategy::Full { reason } => ("full", reason),
        RefreshPlanStrategy::Delta { reason, .. } => ("delta", reason),
    }
}

fn plan_event(
    duration_ms: f64,
    strategy: &str,
    reason: &str,
    changed_count: usize,
    force_full: bool,
    threshold: usize,
) -> Value {
    serde_json::json!({
        "phase": "link_graph.index.delta.plan",
        "duration_ms": duration_ms,
        "extra": {
            "strategy": strategy,
            "reason": reason,
            "changed_count": changed_count,
            "force_full": force_full,
            "threshold": threshold,
            "delta_supported": true,
            "full_refresh_supported": true,
        }
    })
}

fn delta_apply_event(
    duration_ms: f64,
    success: bool,
    changed_count: usize,
    error: Option<&str>,
) -> Value {
    match error {
        Some(message) => serde_json::json!({
            "phase": "link_graph.index.delta.apply",
            "duration_ms": duration_ms,
            "extra": {
                "success": success,
                "changed_count": changed_count,
                "error": message,
            }
        }),
        None => serde_json::json!({
            "phase": "link_graph.index.delta.apply",
            "duration_ms": duration_ms,
            "extra": {
                "success": success,
                "changed_count": changed_count,
            }
        }),
    }
}

fn full_rebuild_event(duration_ms: f64, reason: &str, changed_count: usize) -> Value {
    serde_json::json!({
        "phase": "link_graph.index.rebuild.full",
        "duration_ms": duration_ms,
        "extra": {
            "success": true,
            "reason": reason,
            "changed_count": changed_count,
        }
    })
}

fn serialize_payload(
    mode: &str,
    changed_count: usize,
    force_full: bool,
    fallback: bool,
    events: &[Value],
) -> PyResult<String> {
    let payload = serde_json::json!({
        "mode": mode,
        "changed_count": changed_count,
        "force_full": force_full,
        "fallback": fallback,
        "events": events,
    });
    serde_json::to_string(&payload)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

impl PyLinkGraphEngine {
    fn run_full_refresh_with_events(
        &mut self,
        reason: &'static str,
        changed_count: usize,
        force_full: bool,
        fallback: bool,
        mut events: Vec<Value>,
    ) -> PyResult<String> {
        let full_started = Instant::now();
        self.refresh_impl()?;
        events.push(full_rebuild_event(
            Self::elapsed_ms(full_started),
            reason,
            changed_count,
        ));
        serialize_payload("full", changed_count, force_full, fallback, &events)
    }

    fn run_delta_refresh_with_events(
        &mut self,
        changed_paths: &[PathBuf],
        threshold: usize,
        changed_count: usize,
        mut events: Vec<Value>,
    ) -> PyResult<String> {
        let delta_started = Instant::now();
        match self
            .inner
            .refresh_incremental_with_threshold(changed_paths, threshold)
        {
            Ok(LinkGraphRefreshMode::Noop) => {
                events.push(delta_apply_event(
                    Self::elapsed_ms(delta_started),
                    true,
                    0,
                    None,
                ));
                serialize_payload("noop", 0, false, false, &events)
            }
            Ok(LinkGraphRefreshMode::Delta) => {
                events.push(delta_apply_event(
                    Self::elapsed_ms(delta_started),
                    true,
                    changed_count,
                    None,
                ));
                serialize_payload("delta", changed_count, false, false, &events)
            }
            Ok(LinkGraphRefreshMode::Full) => {
                events.push(full_rebuild_event(
                    Self::elapsed_ms(delta_started),
                    "threshold_exceeded",
                    changed_count,
                ));
                serialize_payload("full", changed_count, false, false, &events)
            }
            Err(delta_error) => {
                events.push(delta_apply_event(
                    Self::elapsed_ms(delta_started),
                    false,
                    changed_count,
                    Some(delta_error.as_str()),
                ));
                self.run_full_refresh_with_events(
                    "delta_failed_fallback",
                    changed_count,
                    false,
                    true,
                    events,
                )
            }
        }
    }

    fn push_plan_event(
        events: &mut Vec<Value>,
        plan_started: Instant,
        strategy: &RefreshPlanStrategy,
        changed_count: usize,
        force_full: bool,
        threshold: usize,
    ) {
        let (strategy_label, reason) = strategy_label_and_reason(strategy);
        events.push(plan_event(
            Self::elapsed_ms(plan_started),
            strategy_label,
            reason,
            changed_count,
            force_full,
            threshold,
        ));
    }

    pub(in crate::pybindings::link_graph_py::engine) fn refresh_plan_apply_impl(
        &mut self,
        changed_paths_json: Option<&str>,
        force_full: bool,
        full_rebuild_threshold: Option<usize>,
    ) -> PyResult<String> {
        let changed_paths = Self::parse_changed_paths(changed_paths_json)?;
        let changed_count = changed_paths.len();
        let threshold = full_rebuild_threshold
            .unwrap_or_else(crate::link_graph::LinkGraphIndex::incremental_rebuild_threshold)
            .max(1);

        let plan_started = Instant::now();
        let strategy = select_refresh_strategy(force_full, changed_count, threshold);
        let mut events = Vec::new();
        Self::push_plan_event(
            &mut events,
            plan_started,
            &strategy,
            changed_count,
            force_full,
            threshold,
        );

        match strategy {
            RefreshPlanStrategy::Noop => serialize_payload("noop", 0, false, false, &events),
            RefreshPlanStrategy::Full { reason } => {
                self.run_full_refresh_with_events(reason, changed_count, force_full, false, events)
            }
            RefreshPlanStrategy::Delta {
                prefer_incremental, ..
            } => {
                let delta_threshold = if prefer_incremental {
                    usize::MAX
                } else {
                    threshold
                };
                self.run_delta_refresh_with_events(
                    &changed_paths,
                    delta_threshold,
                    changed_count,
                    events,
                )
            }
        }
    }
}
