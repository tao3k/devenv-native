//! Internal DMN evaluation seams for engine-owned decision work.

mod evaluate;

pub(crate) use evaluate::evaluate_dmn_decision_sync;
