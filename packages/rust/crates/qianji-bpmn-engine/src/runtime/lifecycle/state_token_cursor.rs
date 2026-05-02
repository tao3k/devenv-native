use crate::runtime::lifecycle::scope::{BpmnInstanceState, PendingHostWork, TokenRecord};
use crate::runtime_instance_api::CallActivityFrame;
use crate::runtime_repeat_api::ParallelMultiInstanceState;

#[derive(Debug, Clone)]
pub(crate) struct TokenIdAllocator {
    next_token_id: u64,
}

impl TokenIdAllocator {
    pub(crate) fn next_token_id(&mut self) -> u64 {
        let token_id = self.next_token_id;
        self.next_token_id = self.next_token_id.saturating_add(1);
        token_id
    }

    pub(crate) fn reserve_on(&self, instance: &mut BpmnInstanceState) {
        reserve_next_token_id(instance, self.next_token_id);
    }
}

pub(crate) fn token_id_allocator(instance: &BpmnInstanceState) -> TokenIdAllocator {
    TokenIdAllocator {
        next_token_id: next_token_id(instance),
    }
}

pub(crate) fn allocate_token_id(instance: &mut BpmnInstanceState) -> u64 {
    let token_id = next_token_id(instance);
    reserve_next_token_id(instance, token_id.saturating_add(1));
    token_id
}

fn next_token_id(instance: &BpmnInstanceState) -> u64 {
    instance
        .next_token_id
        .max(recovered_next_token_id(instance))
}

fn reserve_next_token_id(instance: &mut BpmnInstanceState, next_token_id: u64) {
    instance.next_token_id = instance.next_token_id.max(next_token_id);
}

fn recovered_next_token_id(instance: &BpmnInstanceState) -> u64 {
    instance
        .sequence
        .max(max_token_id(&instance.active_tokens))
        .max(max_pending_token_id(&instance.pending_host_work))
        .max(max_parallel_iteration_token_id(
            &instance.parallel_multi_instances,
        ))
        .max(
            instance
                .call_stack
                .iter()
                .map(max_call_frame_token_id)
                .max()
                .unwrap_or(0),
        )
        .saturating_add(1)
}

fn max_call_frame_token_id(frame: &CallActivityFrame) -> u64 {
    max_token_id(&frame.active_tokens)
        .max(max_pending_token_id(&frame.pending_host_work))
        .max(max_parallel_iteration_token_id(
            &frame.parallel_multi_instances,
        ))
}

fn max_token_id(tokens: &[TokenRecord]) -> u64 {
    tokens.iter().map(|token| token.token_id).max().unwrap_or(0)
}

fn max_pending_token_id(pending: &[PendingHostWork]) -> u64 {
    pending
        .iter()
        .map(|pending_work| pending_work.token_id)
        .max()
        .unwrap_or(0)
}

fn max_parallel_iteration_token_id(parallel_states: &[ParallelMultiInstanceState]) -> u64 {
    parallel_states
        .iter()
        .flat_map(|state| {
            state
                .active_iterations
                .iter()
                .map(|iteration| iteration.token_id)
        })
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "../../../tests/unit/runtime/lifecycle/state_token_cursor.rs"]
mod tests;
