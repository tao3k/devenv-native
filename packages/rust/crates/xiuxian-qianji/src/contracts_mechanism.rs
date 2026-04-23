use async_trait::async_trait;

use super::QianjiOutput;

/// The Holy Trait for every Qianji Mechanism.
///
/// Every implementation serves as an interlocking gear in the Thousand Mechanism box.
#[async_trait]
pub trait QianjiMechanism: Send + Sync {
    /// Executes the core logic of the mechanism with access to the shared context.
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String>;
    /// Returns the scheduling weight/priority.
    fn weight(&self) -> f32; // For probabilistic routing
}
