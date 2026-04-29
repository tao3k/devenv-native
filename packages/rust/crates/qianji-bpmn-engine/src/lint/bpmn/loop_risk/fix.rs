//! Canonical `api` entry for loop-risk repair metadata.

use super::{
    BTreeSet, BpmnProcessSpec, LoopRiskEvidence, ProcessMetadata, Range, Value, is_prompt_output,
    is_state_worker_task, json, sorted_set_values,
};

mod api;
mod fragment;
mod guidance;
mod owner;
mod span;

use fragment::{line_fix, native_input_fragment, native_output_fragment};
use owner::progress_owner_task_id;

pub(super) use api::{
    line_fix_xml_strings, loop_progress_contract_message, loop_progress_help,
    loop_progress_line_fixes, primary_cycle_span,
};
