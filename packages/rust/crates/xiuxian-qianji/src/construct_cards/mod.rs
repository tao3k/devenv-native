//! Construct-card API facade.
//!
//! `api` owns exported DTOs; `catalog` owns card data and `render` owns views.

mod api;
mod catalog;
mod render;
pub use api::{ConstructCard, ConstructIndexEntry, ConstructLintMapping, ConstructStatus};
#[path = "facade.rs"]
mod facade;

pub use facade::{
    construct_cards, construct_index_entries, find_construct_card, render_construct_card,
    render_construct_card_json, render_construct_index, render_construct_index_json,
};
