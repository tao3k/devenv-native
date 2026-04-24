use crate::types::{AuditArgs, Cli};
use anyhow::Result;
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao::zhenfa_router::native::semantic_check::{
    WendaoSemanticCheckArgs, wendao_semantic_check,
};
use xiuxian_zhenfa::ZhenfaContext;

pub(super) fn handle(_cli: &Cli, args: &AuditArgs, index: Option<&LinkGraphIndex>) -> Result<()> {
    let mut ctx = ZhenfaContext::default();

    // Inject the index into context extensions if available
    if let Some(idx) = index {
        ctx.insert_extension(idx.clone());
    } else {
        anyhow::bail!(
            "LinkGraphIndex must be provided for audit (check your environment or --scope)"
        );
    }

    // Convert CLI args to the Tool args
    let check_args = WendaoSemanticCheckArgs {
        doc: Some(args.target.clone()),
        checks: None,
        include_warnings: Some(true),
        source_paths: args.source.as_ref().map(|s| vec![s.clone()]),
        fuzzy_confidence_threshold: Some(args.threshold),
        episteme_load: args.load.clone(),
    };

    let result = wendao_semantic_check(&ctx, check_args)
        .map_err(|e| anyhow::anyhow!("Audit failed: {e:?}"))?;

    println!("{result}");
    Ok(())
}
