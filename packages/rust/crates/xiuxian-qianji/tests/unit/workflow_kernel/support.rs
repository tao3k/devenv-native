pub(super) use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use crate::workflow_kernel::WorkflowStage;
pub(super) use crate::workflow_kernel::{
    WorkflowBoundedFanoutStageRequest, WorkflowCheckpointError, WorkflowCheckpointId,
    WorkflowCheckpointStorageKind, WorkflowCompletionError, WorkflowDuplicateStage,
    WorkflowEdgeKind, WorkflowMemoryCheckpointRecord, WorkflowRun, WorkflowStageBinding,
    WorkflowStageCheckpointMiss, WorkflowStageFacts, WorkflowStageId, WorkflowStageStatus,
    WorkflowTopology, WorkflowTopologyEdge, WorkflowTopologyError,
};

#[derive(Debug, Default)]
pub(super) struct TestContext {
    pub(super) events: Vec<&'static str>,
}

#[derive(Debug)]
pub(super) struct AppendStage {
    pub(super) id: &'static str,
    pub(super) suffix: &'static str,
}

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for AppendStage {
    type Output = String;
    type Error = String;

    fn id(&self) -> &'static str {
        self.id
    }

    fn input_facts(&self, input: &String) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(input.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(output.len())
    }

    async fn run(&self, context: &mut TestContext, input: String) -> Result<Self::Output, String> {
        context.events.push(self.id);
        Ok(format!("{input}{}", self.suffix))
    }
}

#[derive(Debug)]
pub(super) struct LenStage;

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for LenStage {
    type Output = usize;
    type Error = String;

    fn id(&self) -> &'static str {
        "len"
    }

    fn input_facts(&self, input: &String) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(input.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("usize").with_item_count(*output)
    }

    async fn run(&self, context: &mut TestContext, input: String) -> Result<Self::Output, String> {
        context.events.push("len");
        Ok(input.len())
    }
}

#[derive(Debug)]
pub(super) struct FailingStage;

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for FailingStage {
    type Output = String;
    type Error = String;

    fn id(&self) -> &'static str {
        "fail"
    }

    async fn run(&self, context: &mut TestContext, _input: String) -> Result<Self::Output, String> {
        context.events.push("fail");
        Err("intentional failure".to_owned())
    }
}

pub(super) fn assert_err<T, E>(result: Result<T, E>, message: &str) -> E {
    let Err(error) = result else {
        panic!("{message}");
    };
    error
}
