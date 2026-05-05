/// One required startup dependency check for the Studio gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayStartupDependencyCheck {
    /// Stable dependency identifier used in logs and failure summaries.
    pub dependency: &'static str,
    /// Health status for the dependency.
    pub status: GatewayStartupDependencyStatus,
    /// Human-readable resolution detail.
    pub detail: String,
}

impl GatewayStartupDependencyCheck {
    /// Construct a connected dependency check.
    #[must_use]
    pub fn connected(dependency: &'static str, detail: impl Into<String>) -> Self {
        Self {
            dependency,
            status: GatewayStartupDependencyStatus::Connected,
            detail: detail.into(),
        }
    }

    /// Construct a failed dependency check.
    #[must_use]
    pub fn failed(dependency: &'static str, detail: impl Into<String>) -> Self {
        Self {
            dependency,
            status: GatewayStartupDependencyStatus::Failed,
            detail: detail.into(),
        }
    }
}

/// Gateway startup dependency health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayStartupDependencyStatus {
    /// Dependency is healthy and connected.
    Connected,
    /// Dependency failed validation or connectivity.
    Failed,
}

impl GatewayStartupDependencyStatus {
    /// Stable label used in startup logs.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Failed => "failed",
        }
    }
}

/// Startup-health summary for all required gateway dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayStartupHealthReport {
    checks: Vec<GatewayStartupDependencyCheck>,
}

impl GatewayStartupHealthReport {
    /// Build a startup-health report from ordered dependency checks.
    #[must_use]
    pub fn new(checks: Vec<GatewayStartupDependencyCheck>) -> Self {
        Self { checks }
    }

    /// Ordered dependency checks included in the startup gate.
    #[must_use]
    pub fn checks(&self) -> &[GatewayStartupDependencyCheck] {
        self.checks.as_slice()
    }

    /// Whether all required dependencies are healthy.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.checks
            .iter()
            .all(|check| check.status == GatewayStartupDependencyStatus::Connected)
    }

    /// Human-readable failure summary for startup aborts.
    #[must_use]
    pub fn failure_summary(&self) -> Option<String> {
        let failures = self
            .checks
            .iter()
            .filter(|check| check.status == GatewayStartupDependencyStatus::Failed)
            .map(|check| format!("{} ({})", check.dependency, check.detail))
            .collect::<Vec<_>>();
        (!failures.is_empty()).then(|| failures.join("; "))
    }
}
