use crate::error::{BpmnEngineError, Result};
use crate::ir_repeat_api::{
    BpmnMultiInstanceDataBindingSpec, BpmnParallelMultiInstanceSpec, BpmnRepeatSpec,
    BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec,
};
use crate::parser::import::{
    RawNode, RawParallelMultiInstanceSpec, RawProcess, RawRepeatSpec,
    RawSequentialMultiInstanceSpec,
};

pub(super) fn normalize_repeat_spec(
    raw: &RawProcess,
    node: &RawNode,
    spec: crate::ir_node_api::BpmnNodeSpec,
) -> Result<crate::ir_node_api::BpmnNodeSpec> {
    match &node.repeat {
        Some(RawRepeatSpec::StandardLoop(loop_spec)) => {
            let repeat = BpmnStandardLoopSpec::new(loop_spec.test_before, loop_spec.loop_maximum);
            let repeat = match &loop_spec.loop_condition {
                Some(loop_condition) => repeat.with_loop_condition(loop_condition),
                None => repeat,
            };
            Ok(spec.with_repeat(BpmnRepeatSpec::StandardLoop(repeat)))
        }
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            let repeat = if let Some(loop_cardinality) = loop_spec.loop_cardinality {
                BpmnSequentialMultiInstanceSpec::new(loop_cardinality)
            } else {
                BpmnSequentialMultiInstanceSpec::from_data_binding(
                    normalize_multi_instance_data_binding(raw, node, loop_spec)?,
                )
            };
            let repeat = match &loop_spec.completion_condition {
                Some(completion_condition) => {
                    repeat.with_completion_condition(completion_condition)
                }
                None => repeat,
            };
            Ok(spec.with_repeat(BpmnRepeatSpec::SequentialMultiInstance(repeat)))
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            let repeat = if let Some(loop_cardinality) = loop_spec.loop_cardinality {
                BpmnParallelMultiInstanceSpec::new(loop_cardinality)
            } else {
                BpmnParallelMultiInstanceSpec::from_data_binding(
                    normalize_multi_instance_data_binding(raw, node, loop_spec)?,
                )
            };
            let repeat = match &loop_spec.completion_condition {
                Some(completion_condition) => {
                    repeat.with_completion_condition(completion_condition)
                }
                None => repeat,
            };
            Ok(spec.with_repeat(BpmnRepeatSpec::ParallelMultiInstance(repeat)))
        }
        None => Ok(spec),
    }
}

fn normalize_multi_instance_data_binding(
    raw: &RawProcess,
    node: &RawNode,
    loop_spec: &impl RawMultiInstanceDataBindingFields,
) -> Result<BpmnMultiInstanceDataBindingSpec> {
    let loop_data_input_ref = loop_spec.loop_data_input_ref().ok_or_else(|| {
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: (raw.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "missing_loop_cardinality_or_data_input",
        }
    })?;
    let input_data_item = loop_spec.input_data_item().ok_or_else(|| {
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: (raw.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "missing_input_data_item",
        }
    })?;
    let binding = BpmnMultiInstanceDataBindingSpec::new(loop_data_input_ref, input_data_item);
    match (
        loop_spec.loop_data_output_ref(),
        loop_spec.output_data_item(),
    ) {
        (Some(loop_data_output_ref), Some(output_data_item)) => {
            Ok(binding.with_output(loop_data_output_ref, output_data_item))
        }
        (None, None) => Ok(binding),
        (Some(_), None) => Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: (raw.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "missing_output_data_item",
        }),
        (None, Some(_)) => Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: (raw.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "missing_loop_data_output_ref",
        }),
    }
}

trait RawMultiInstanceDataBindingFields {
    fn loop_data_input_ref(&self) -> Option<&str>;
    fn input_data_item(&self) -> Option<&str>;
    fn loop_data_output_ref(&self) -> Option<&str>;
    fn output_data_item(&self) -> Option<&str>;
}

impl RawMultiInstanceDataBindingFields for RawSequentialMultiInstanceSpec {
    fn loop_data_input_ref(&self) -> Option<&str> {
        self.loop_data_input_ref.as_deref()
    }

    fn input_data_item(&self) -> Option<&str> {
        self.input_data_item.as_deref()
    }

    fn loop_data_output_ref(&self) -> Option<&str> {
        self.loop_data_output_ref.as_deref()
    }

    fn output_data_item(&self) -> Option<&str> {
        self.output_data_item.as_deref()
    }
}

impl RawMultiInstanceDataBindingFields for RawParallelMultiInstanceSpec {
    fn loop_data_input_ref(&self) -> Option<&str> {
        self.loop_data_input_ref.as_deref()
    }

    fn input_data_item(&self) -> Option<&str> {
        self.input_data_item.as_deref()
    }

    fn loop_data_output_ref(&self) -> Option<&str> {
        self.loop_data_output_ref.as_deref()
    }

    fn output_data_item(&self) -> Option<&str> {
        self.output_data_item.as_deref()
    }
}
