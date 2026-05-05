//! Seed one minimal repo-search sample corpus into the Wendao search plane.

#[cfg(not(feature = "julia"))]
fn main() {
    eprintln!("wendao_search_seed_sample requires the `julia` feature");
    std::process::exit(1);
}

#[cfg(feature = "julia")]
use std::env;
#[cfg(feature = "julia")]
use std::path::PathBuf;

#[cfg(feature = "julia")]
use anyhow::{Result, anyhow};
#[cfg(feature = "julia")]
use xiuxian_wendao::search::SearchPlaneService;
#[cfg(feature = "julia")]
use xiuxian_wendao_studio::studio::bootstrap_sample_repo_search_content;

#[cfg(feature = "julia")]
#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let repo_id = args.next().unwrap_or_else(|| "alpha/repo".to_string());
    let project_root = match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            env::current_dir().map_err(|error| anyhow!("failed to resolve current dir: {error}"))?
        }
    };

    let service = SearchPlaneService::new(project_root);
    bootstrap_sample_repo_search_content(&service, repo_id.as_str())
        .await
        .map_err(|error| anyhow!(error))?;
    println!("SEEDED {repo_id}");
    Ok(())
}
