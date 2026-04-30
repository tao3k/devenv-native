#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawRepeatSpec {
    StandardLoop(RawStandardLoopSpec),
    SequentialMultiInstance(RawSequentialMultiInstanceSpec),
    ParallelMultiInstance(RawParallelMultiInstanceSpec),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawStandardLoopSpec {
    pub(crate) test_before: bool,
    pub(crate) loop_maximum: Option<u32>,
    pub(crate) loop_condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSequentialMultiInstanceSpec {
    pub(crate) loop_cardinality: Option<u32>,
    pub(crate) loop_data_input_ref: Option<String>,
    pub(crate) input_data_item: Option<String>,
    pub(crate) loop_data_output_ref: Option<String>,
    pub(crate) output_data_item: Option<String>,
    pub(crate) completion_condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawParallelMultiInstanceSpec {
    pub(crate) loop_cardinality: Option<u32>,
    pub(crate) loop_data_input_ref: Option<String>,
    pub(crate) input_data_item: Option<String>,
    pub(crate) loop_data_output_ref: Option<String>,
    pub(crate) output_data_item: Option<String>,
    pub(crate) completion_condition: Option<String>,
}
