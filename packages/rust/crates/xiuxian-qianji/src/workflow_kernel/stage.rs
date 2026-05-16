//! Workflow stage trait.

use super::WorkflowStageFacts;

/// Strongly typed stage boundary for Rust-native workflow execution.
#[async_trait::async_trait]
pub trait WorkflowStage<C, I>: Send + Sync
where
    C: Send,
    I: Send,
{
    /// Stage output type.
    type Output: Send;
    /// Stage error type.
    type Error: std::fmt::Display + Send + Sync + 'static;

    /// Stable stage identifier used in traces and future graph bindings.
    fn id(&self) -> &'static str;

    /// Captures input facts for tracing before stage execution.
    fn input_facts(&self, _input: &I) -> WorkflowStageFacts {
        WorkflowStageFacts::default()
    }

    /// Captures output facts for tracing after successful stage execution.
    fn output_facts(&self, _output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::default()
    }

    /// Executes this stage.
    ///
    /// # Errors
    ///
    /// Returns the stage-owned error when the stage cannot produce its output.
    async fn run(&self, context: &mut C, input: I) -> Result<Self::Output, Self::Error>;
}
