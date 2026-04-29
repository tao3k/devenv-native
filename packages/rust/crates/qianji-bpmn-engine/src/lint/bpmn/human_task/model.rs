use std::ops::Range;

pub(super) struct ProcessContext {
    pub(super) process_id: Option<String>,
}

#[derive(Clone)]
pub(super) struct HumanTaskContext {
    pub(super) task_id: Option<String>,
    pub(super) task_kind: String,
}

#[derive(Clone)]
pub(super) struct GlobalHumanTaskContext {
    pub(super) task_id: String,
    pub(super) task_kind: String,
}

#[derive(Clone)]
pub(super) struct CallActivityContext {
    pub(super) process_id: Option<String>,
    pub(super) activity_id: Option<String>,
    pub(super) called_element: String,
    pub(super) span: Range<usize>,
}
