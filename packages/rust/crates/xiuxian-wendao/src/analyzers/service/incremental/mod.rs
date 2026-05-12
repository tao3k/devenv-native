//! `analyzers::service::incremental` owns Wendao analyzers service incremental behavior.

mod merge;
mod orchestration;
mod relations;

pub(crate) use orchestration::{
    IncrementalApplyContext, analyze_changed_files, apply_incremental_plugin_outputs,
};
