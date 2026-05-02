//! Agent memory branch for lifecycle, feedback, and storage operations.

mod decay;
mod recall_credit;

pub(crate) use decay::{sanitize_decay_factor, should_apply_decay};
pub(crate) use recall_credit::{
    RecallCreditUpdate, RecalledEpisodeCandidate, apply_recall_credit,
    select_recall_credit_candidates,
};
