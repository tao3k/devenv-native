use crate::bin_support::wendao::types::{Cli, FixArgs};
use crate::link_graph::LinkGraphIndex;
use crate::zhenfa_router::native::audit::fix::{AtomicFixBatch, format_fix_preview};
use crate::zhenfa_router::native::audit::generate_surgical_fixes;
use crate::zhenfa_router::native::semantic_check::{WendaoSemanticCheckArgs, run_audit_core};
use anyhow::Result;
use xiuxian_zhenfa::ZhenfaContext;

pub(super) fn handle(_cli: &Cli, args: &FixArgs, index: Option<&LinkGraphIndex>) -> Result<()> {
    let mut ctx = ZhenfaContext::default();

    // Inject the index into context extensions
    if let Some(idx) = index {
        ctx.insert_extension(idx.clone());
    } else {
        anyhow::bail!("LinkGraphIndex must be provided for fix (check your environment or --root)");
    }

    println!("🔍 Phase 1: Auditing targets for remediable issues...");

    // 1. Run the official semantic check core
    let check_args = WendaoSemanticCheckArgs {
        doc: Some(args.path.clone()),
        checks: None,
        include_warnings: Some(true),
        source_paths: Some(vec!["src".to_string()]),
        fuzzy_confidence_threshold: Some(args.confidence_threshold),
        episteme_load: None,
    };

    let (issues, file_contents) = run_audit_core(&ctx, &check_args)
        .map_err(|e| anyhow::anyhow!("Audit core failed: {e:?}"))?;

    if issues.is_empty() {
        println!("✅ No issues found. Your knowledge base is healthy.");
        return Ok(());
    }

    // 2. Generate surgical fixes
    println!("🛠️  Phase 2: Generating surgical remediation plans...");
    let mut fixes = generate_surgical_fixes(&issues, &file_contents);

    if let Some(issue_type) = &args.issue_type {
        fixes.retain(|fix| fix.issue_type == *issue_type);
    }

    fixes.retain(|fix| fix.confidence >= args.confidence_threshold);

    if let Some(idx) = index {
        for fix in &mut fixes {
            if let Some(path) = idx.doc_path(&fix.doc_path) {
                fix.doc_path = path.to_string();
            }
        }
    }

    if fixes.is_empty() {
        println!("⚠️  Found issues, but none have automatic fix suggestions.");
        return Ok(());
    }

    // 3. Apply fixes
    let batch = AtomicFixBatch::new(fixes);

    if args.dry_run {
        let previews = batch.preview_all();
        if previews.is_empty() {
            println!("🧪 Dry Run: no previewable fixes were generated.");
        } else {
            println!(
                "🧪 Dry Run: Previewing {} suggested fixes:",
                batch.total_fixes()
            );
            println!("{}", format_fix_preview(&previews));
        }
        println!("\nTotal: {} fixes (NOT APPLIED)", batch.total_fixes());
    } else {
        println!("🚀 Applying {} surgical fixes...", batch.total_fixes());
        let report = batch.apply_all();
        println!("{}", report.summary());
        if !report.errors.is_empty() {
            for error in &report.errors {
                eprintln!("  ❌ {error}");
            }
        }
        if !report.is_success() {
            anyhow::bail!("One or more fixes could not be applied");
        }
        println!("\n✅ Remediation complete.");
    }

    Ok(())
}
