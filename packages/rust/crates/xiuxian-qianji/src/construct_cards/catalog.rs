//! Static construct-card registry split by construct family.
//!
//! Start with `registry`; sibling modules own construct-specific cards.

use crate::construct_cards::{ConstructCard, ConstructLintMapping};

mod agent;
mod dmn;
mod gateway;
mod interaction;
mod loop_progress;
mod multi_instance;
mod registry;

pub(crate) use registry::{construct_cards, construct_index_entries, find_construct_card};
