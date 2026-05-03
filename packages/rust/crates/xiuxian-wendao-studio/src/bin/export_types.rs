//! Export TypeScript bindings from Rust Specta types.
//!
//! This binary generates `bindings.ts` for the Wendao frontend,
//! ensuring type safety between the Rust backend and TypeScript frontend.
//!
//! Usage:
//!   `cargo run --bin export_types --features studio`

use specta_typescript::{BigIntExportBehavior, Typescript};
use xiuxian_wendao::search::contracts::studio_frontend_type_collection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let types = studio_frontend_type_collection();
    let ts = Typescript::new()
        .bigint(BigIntExportBehavior::Number)
        .header("// Auto-generated from xiuxian-wendao\n// Run: cargo run --bin export_types --features studio\n\n")
        .export(&types)?;

    let output_path =
        std::path::PathBuf::from(".data/wendao-frontend/src/api/studioBindings.generated.ts");

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, ts)?;
    println!("TypeScript bindings written to: {}", output_path.display());
    Ok(())
}
