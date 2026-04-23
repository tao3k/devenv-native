//! Application-layer scheduler factories and built-in pipeline presets.

mod build;
#[path = "../app_presets.rs"]
mod presets;
#[path = "../app_qianji_app.rs"]
mod qianji_app;

pub use presets::{
    MEMORY_PROMOTION_PIPELINE_TOML, RESEARCH_TRINITY_TOML, WENDAO_SQL_AUTHORING_V1_TOML,
};
pub use qianji_app::QianjiApp;
