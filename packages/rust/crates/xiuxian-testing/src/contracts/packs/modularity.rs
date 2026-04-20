//! Deterministic modularity checks over Rust source artifacts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::contracts::model::{
    ArtifactKind, CollectedArtifact, CollectedArtifacts, CollectionContext, ContractFinding,
    EvidenceKind, FindingEvidence, FindingMode, FindingSeverity,
};
use crate::contracts::rule_pack::{RulePack, RulePackDescriptor};

use self::file_bloat::FileBloatCheck;
use self::relative_import::RelativeImportCheck;
use self::root_alias::RootFacadeAliasCheck;
use self::root_child_visibility::RootChildVisibilityCheck;
use self::root_doc_curation::RootDocCurationCheck;
use self::root_doc_hint::RootDocHintCheck;
use self::root_doc_owner_alignment::RootDocOwnerAlignmentCheck;
use self::root_doc_owner_name::RootDocOwnerNameCheck;
use self::root_doc_priority::RootDocPriorityCheck;
use self::root_doc_secondary_budget::RootDocSecondaryBudgetCheck;
use self::root_doc_secondary_owner::RootDocSecondaryOwnerCheck;
use self::root_entry_curation::RootEntryCurationCheck;
use self::root_entry_focus::RootEntryFocusCheck;
use self::root_entry_owner::RootEntryOwnerCheck;
use self::root_entry_visibility::RootEntryVisibilityCheck;
use self::root_facade::RootFacadeExportCheck;
use self::root_hint::RootNavigationHintCheck;
use self::root_owner_convergence::RootOwnerConvergenceCheck;
use self::root_toc::RootModuleTocCheck;
use self::rust_syntax::ModInterfaceCheck;

mod file_bloat;
mod relative_import;
mod root_alias;
mod root_child_visibility;
mod root_doc_curation;
mod root_doc_hint;
mod root_doc_owner_alignment;
mod root_doc_owner_name;
mod root_doc_priority;
mod root_doc_secondary_budget;
mod root_doc_secondary_owner;
mod root_entry_curation;
mod root_entry_focus;
mod root_entry_owner;
mod root_entry_visibility;
mod root_facade;
mod root_hint;
mod root_owner_convergence;
mod root_toc;
mod rust_syntax;

const PACK_ID: &str = "modularity";
const MOD_R001: &str = "MOD-R001";
const MOD_R002: &str = "MOD-R002";
const MOD_R003: &str = "MOD-R003";
const MOD_R006: &str = "MOD-R006";
const MOD_R007: &str = "MOD-R007";
const MOD_R008: &str = "MOD-R008";
const MOD_R009: &str = "MOD-R009";
const MOD_R010: &str = "MOD-R010";
const MOD_R011: &str = "MOD-R011";
const MOD_R012: &str = "MOD-R012";
const MOD_R013: &str = "MOD-R013";
const MOD_R014: &str = "MOD-R014";
const MOD_R015: &str = "MOD-R015";
const MOD_R016: &str = "MOD-R016";
const MOD_R017: &str = "MOD-R017";
const MOD_R018: &str = "MOD-R018";
const MOD_R019: &str = "MOD-R019";
const MOD_R020: &str = "MOD-R020";
const MOD_R021: &str = "MOD-R021";
const MOD_R022: &str = "MOD-R022";
const MOD_R023: &str = "MOD-R023";
const MOD_R024: &str = "MOD-R024";

/// Deterministic V1 modularity checks over Rust source files.
#[derive(Debug, Default, Clone, Copy)]
pub struct ModularityRulePack;

impl RulePack for ModularityRulePack {
    fn descriptor(&self) -> RulePackDescriptor {
        RulePackDescriptor {
            id: PACK_ID,
            version: "v1",
            domains: &["modularity", "architecture", "rust"],
            default_mode: FindingMode::Deterministic,
        }
    }

    fn collect(&self, ctx: &CollectionContext) -> Result<CollectedArtifacts> {
        let Some(source_root) = resolve_source_root(ctx) else {
            return Ok(CollectedArtifacts::default());
        };

        let mut files = Vec::new();
        collect_rust_files(&source_root, &mut files)?;
        files.sort();

        let mut artifacts = CollectedArtifacts::default();
        artifacts
            .metadata
            .insert("source_root".to_string(), source_root.display().to_string());
        for file in files {
            let text = fs::read_to_string(&file)
                .with_context(|| format!("failed to read Rust source file {}", file.display()))?;
            artifacts.push(CollectedArtifact {
                id: artifact_id(ctx.workspace_root.as_deref(), &file),
                kind: ArtifactKind::SourceFile,
                path: Some(file),
                content: json!({ "text": text }),
                labels: BTreeMap::from([("pack".to_string(), PACK_ID.to_string())]),
            });
        }

        Ok(artifacts)
    }

    fn evaluate(&self, artifacts: &CollectedArtifacts) -> Result<Vec<ContractFinding>> {
        let mut findings = Vec::new();
        let file_texts = artifacts
            .artifacts
            .iter()
            .filter_map(|artifact| {
                let path = artifact.path.as_ref()?;
                let text = artifact.content.get("text").and_then(Value::as_str)?;
                Some((path.clone(), text.to_string()))
            })
            .collect::<BTreeMap<_, _>>();
        for artifact in &artifacts.artifacts {
            if artifact.kind != ArtifactKind::SourceFile {
                continue;
            }
            let Some(path) = artifact.path.as_deref() else {
                continue;
            };
            let Some(text) = artifact.content.get("text").and_then(Value::as_str) else {
                continue;
            };

            findings.extend(check_mod_interface_only(path, text));
            findings.extend(check_file_bloat(path, text));
            findings.extend(check_root_module_toc(path, text));
            findings.extend(check_root_facade_exports(path, text));
            findings.extend(check_root_facade_aliases(path, text));
            findings.extend(check_root_navigation_hint(path, text));
            findings.extend(check_root_doc_hint_specificity(path, text));
            findings.extend(check_root_doc_owner_alignment(path, text));
            findings.extend(check_root_entry_focus(path, text));
            findings.extend(check_root_owner_convergence(path, text));
            findings.extend(check_root_entry_visibility(path, text, &file_texts));
            findings.extend(check_root_entry_curation(path, text, &file_texts));
            findings.extend(check_root_doc_curation(path, text, &file_texts));
            findings.extend(check_root_doc_priority(path, text, &file_texts));
            findings.extend(check_root_doc_secondary_budget(path, text, &file_texts));
            findings.extend(check_root_doc_secondary_owner(path, text, &file_texts));
            findings.extend(check_root_doc_owner_name(path, text, &file_texts));
            findings.extend(check_root_entry_owner(path, text));
            findings.extend(check_root_child_visibility(path, text));
            findings.extend(check_relative_import_clarity(path, text));
            findings.extend(check_visibility_boundary(path, text));
            findings.extend(check_public_result_error_docs(path, text));
        }
        Ok(findings)
    }
}

fn resolve_source_root(ctx: &CollectionContext) -> Option<PathBuf> {
    let workspace_root = ctx.workspace_root.as_ref()?;
    if let Some(crate_name) = ctx
        .crate_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        let canonical = workspace_root
            .join("packages")
            .join("rust")
            .join("crates")
            .join(crate_name)
            .join("src");
        if canonical.is_dir() {
            return Some(canonical);
        }

        let fallback = workspace_root.join(crate_name).join("src");
        if fallback.is_dir() {
            return Some(fallback);
        }
    }

    let workspace_src = workspace_root.join("src");
    if workspace_src.is_dir() {
        return Some(workspace_src);
    }
    None
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(root)
        .with_context(|| format!("failed to read source directory {}", root.display()))?;
    for entry in entries {
        let entry = entry
            .with_context(|| format!("failed to traverse source directory {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files)?;
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn artifact_id(workspace_root: Option<&Path>, path: &Path) -> String {
    if let Some(root) = workspace_root {
        return path.strip_prefix(root).ok().map_or_else(
            || path.to_string_lossy().into_owned(),
            |relative| relative.to_string_lossy().into_owned(),
        );
    }
    path.to_string_lossy().into_owned()
}

fn check_mod_interface_only(path: &Path, text: &str) -> Vec<ContractFinding> {
    if path.file_name().and_then(|name| name.to_str()) != Some("mod.rs") {
        return Vec::new();
    }

    match rust_syntax::check_mod_rs_interface(text) {
        ModInterfaceCheck::InterfaceOnly => Vec::new(),
        ModInterfaceCheck::NonInterfaceItem(item) => {
            let mut finding = base_finding(
                MOD_R001,
                FindingSeverity::Error,
                path,
                "mod.rs should remain interface-only",
                format!(
                    "`{}` contains {} at line {}.",
                    display_path(path),
                    item.description,
                    item.line_number
                ),
            );
            finding.why_it_matters = "Interface modules should expose structure only; implementation in `mod.rs` blurs module boundaries.".to_string();
            finding.remediation = "Move implementation into dedicated submodules and keep `mod.rs` to private `mod child;` or private `#[path = \"../child.rs\"] mod child;` declarations plus explicit `pub use`/`pub(crate) use` re-exports. Do not publish child modules directly from `mod.rs`.".to_string();
            finding
                .examples
                .good
                .push("`mod.rs` with private `mod foo;` or private `#[path = \"../foo.rs\"] mod foo;` plus a selective `pub use foo::Type;` entry export only.".to_string());
            finding.examples.bad.push(
                "`mod.rs` defines business functions, exposes `pub mod`, uses glob re-exports, or carries concrete state.".to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                item.line_number,
                format!("Top-level {} found in `mod.rs`.", item.description),
            ));
            vec![finding]
        }
        ModInterfaceCheck::ParseFailure(error) => {
            let mut finding = base_finding(
                MOD_R001,
                FindingSeverity::Error,
                path,
                "mod.rs should remain interface-only",
                format!(
                    "`{}` could not be parsed as Rust syntax at line {}, column {} while verifying the interface-only module contract.",
                    display_path(path),
                    error.line_number,
                    error.column_number
                ),
            );
            finding.why_it_matters = "The interface-only contract must be proven from valid Rust syntax; parse failures hide ownership drift and block reliable modularity auditing.".to_string();
            finding.remediation = "Fix the Rust syntax error, then keep `mod.rs` as an interface-only seam with private `mod child;` or private `#[path = \"../child.rs\"] mod child;` declarations plus explicit re-exports. Move concrete implementation out to sibling owner files before re-running the gate.".to_string();
            finding
                .examples
                .good
                .push("A syntactically valid `mod.rs` that contains only private module declarations, optional private `#[path = ...]` mounts, and explicit re-exports.".to_string());
            finding
                .examples
                .bad
                .push("A `mod.rs` file that does not parse, so the module-boundary contract cannot be proven.".to_string());
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                error.line_number,
                format!(
                    "Rust syntax parse failed at column {}: {}",
                    error.column_number, error.message
                ),
            ));
            vec![finding]
        }
    }
}

fn check_file_bloat(path: &Path, text: &str) -> Vec<ContractFinding> {
    match file_bloat::check_rust_file_bloat(path, text) {
        FileBloatCheck::WithinBounds | FileBloatCheck::Skipped => Vec::new(),
        FileBloatCheck::Bloated(metrics) => {
            let mut finding = base_finding(
                MOD_R006,
                FindingSeverity::Warning,
                path,
                "Rust source file appears too large for one ownership seam",
                format!(
                    "`{}` has {} effective code lines, {} top-level items, {} responsibility groups, and {} top-level public-surface items.",
                    display_path(path),
                    metrics.effective_code_lines,
                    metrics.top_level_items,
                    metrics.responsibility_groups,
                    metrics.public_surface_items
                ),
            );
            finding.why_it_matters = "Rust code stays easier for both humans and coding agents to navigate when one file owns one bounded concern instead of becoming a mixed-responsibility sink. A wider sibling-file layout is acceptable when the folder boundary remains clear.".to_string();
            finding.remediation = "Split the file by responsibility into focused sibling modules or a feature folder. Prefer a clear root module or facade that points coding agents at the right files; do not treat additional sibling files as a problem by themselves.".to_string();
            finding.examples.good.push(
                "`feature/` with focused `types.rs`, `service.rs`, `parser.rs`, and `runtime.rs`, plus a small root module that acts as the table of contents."
                    .to_string(),
            );
            finding.examples.bad.push(
                "One large `feature.rs` mixing data types, runtime flow, parsing, public API glue, and many helpers because the file was never split."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                1,
                format!(
                    "effective_code_lines={}, top_level_items={}, responsibility_groups={}, public_surface_items={}",
                    metrics.effective_code_lines,
                    metrics.top_level_items,
                    metrics.responsibility_groups,
                    metrics.public_surface_items
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_module_toc(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_toc::check_root_module_toc(path, text) {
        RootModuleTocCheck::NotApplicable | RootModuleTocCheck::ClearToc => Vec::new(),
        RootModuleTocCheck::MixedRoot(metrics) => {
            let mut finding = base_finding(
                MOD_R007,
                FindingSeverity::Warning,
                path,
                "Root module should stay as a navigational table of contents",
                format!(
                    "`{}` declares {} child modules but still contains {} effective code lines and top-level implementation item `{}` at line {}.",
                    display_path(path),
                    metrics.child_modules,
                    metrics.effective_code_lines,
                    metrics.first_implementation_item,
                    metrics.first_implementation_line
                ),
            );
            finding.why_it_matters = "When a root module already fans out into several sibling modules, keeping that root focused as a table of contents makes the feature easier for both humans and coding agents to navigate. The problem is not sibling-file fan-out; the problem is losing the root seam to mixed implementation.".to_string();
            finding.remediation = "Keep the root module as a small facade or table of contents, and move implementation into focused sibling files under the feature folder. A higher number of leaf files is acceptable when the root still points clearly at them.".to_string();
            finding.examples.good.push(
                "`feature.rs` or `feature/mod.rs` that mostly declares child modules and selective re-exports while sibling files hold the implementation."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`feature.rs` that declares several child modules and still carries parsing, runtime flow, helper logic, or stateful types in the same root file."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.first_implementation_line,
                format!(
                    "child_modules={}, effective_code_lines={}, first_implementation_item={}",
                    metrics.child_modules,
                    metrics.effective_code_lines,
                    metrics.first_implementation_item
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_facade_exports(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_facade::check_root_facade_exports(path, text) {
        RootFacadeExportCheck::NotApplicable | RootFacadeExportCheck::CuratedFacade => Vec::new(),
        RootFacadeExportCheck::NoisyFacade(metrics) => {
            let mut finding = base_finding(
                MOD_R008,
                FindingSeverity::Warning,
                path,
                "Root facade export surface looks too wide",
                format!(
                    "`{}` declares {} child modules and re-exports {} symbols from {} child-module surfaces at the root facade.",
                    display_path(path),
                    metrics.child_modules,
                    metrics.exported_symbols,
                    metrics.source_modules
                ),
            );
            finding.why_it_matters = "A root facade is easiest for coding agents to use when it exposes a small curated entry surface instead of forwarding most leaf symbols. The issue is not sibling-file fan-out; the issue is making the root facade too noisy to serve as the primary navigation seam.".to_string();
            finding.remediation = "Keep the root facade selective: re-export the canonical entry points and let detailed helper types stay in leaf modules. Codex or Claude can descend into sibling files once the top-level seam makes the main ownership path obvious.".to_string();
            finding.examples.good.push(
                "A root facade that re-exports one or two canonical entry points per child module while the leaf files retain detailed helper types."
                    .to_string(),
            );
            finding.examples.bad.push(
                "A root facade that forwards nearly every parser, runtime, and service type so the top-level seam becomes a flat export list."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.first_export_line,
                format!(
                    "child_modules={}, exported_symbols={}, source_modules={}",
                    metrics.child_modules, metrics.exported_symbols, metrics.source_modules
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_facade_aliases(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_alias::check_root_facade_aliases(path, text) {
        RootFacadeAliasCheck::NotApplicable | RootFacadeAliasCheck::ClearFacade => Vec::new(),
        RootFacadeAliasCheck::PublicAlias(alias) => {
            let mut finding = base_finding(
                MOD_R010,
                FindingSeverity::Warning,
                path,
                "Root facade should avoid public alias re-exports",
                format!(
                    "`{}` re-exports `{}` as `{}` at line {}. Keep the root facade aligned with the canonical owner path when possible.",
                    display_path(path),
                    alias.source_symbol,
                    alias.alias_name,
                    alias.line_number
                ),
            );
            finding.why_it_matters = "Public alias re-exports in a root facade make coding agents learn one name at the entry seam and a different name in the owner module. That obscures the real ownership path even when the folder structure is otherwise clear.".to_string();
            finding.remediation = "Prefer re-exporting the canonical symbol name at the root facade. Reserve aliasing for narrower internal visibility or cases where the rename is intentionally part of the crate's public contract.".to_string();
            finding.examples.good.push(
                "`pub use self::parser::Parser;` keeps the top-level seam aligned with the leaf owner."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`pub use self::parser::Parser as FeatureParser;` makes the root seam and the owner module disagree on the symbol name."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                alias.line_number,
                alias.rendered_path,
            ));
            vec![finding]
        }
    }
}

fn check_root_navigation_hint(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_hint::check_root_navigation_hint(path, text) {
        RootNavigationHintCheck::NotApplicable | RootNavigationHintCheck::HintPresent => Vec::new(),
        RootNavigationHintCheck::MissingHint(metrics) => {
            let mut finding = base_finding(
                MOD_R011,
                FindingSeverity::Warning,
                path,
                "Root seam should provide a navigation hint",
                format!(
                    "`{}` declares {} child modules but provides no root-level doc hint or visible re-export for the feature seam.",
                    display_path(path),
                    metrics.child_modules
                ),
            );
            finding.why_it_matters = "When a feature already fans out across several sibling modules, coding agents benefit from one explicit hint at the root seam: a short root doc or a small visible re-export surface. Without that first signal, the agent has to guess which leaf file to open first.".to_string();
            finding.remediation = "Add one root-level `//!` doc that explains the feature seam, or expose a small visible re-export such as the canonical entry type or service. The goal is to point Codex at the first hop, not to expand the public surface indiscriminately.".to_string();
            finding.examples.good.push(
                "`//! Parser + runtime seam for feature X.` plus focused child modules, or a small `pub(crate) use self::service::Service;` entry export."
                    .to_string(),
            );
            finding.examples.bad.push(
                "A root file with several `mod` declarations and no doc hint or visible entry export, leaving the first hop ambiguous."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                1,
                format!("child_modules={}", metrics.child_modules),
            ));
            vec![finding]
        }
    }
}

fn check_root_entry_owner(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_entry_owner::check_root_entry_owner(path, text) {
        RootEntryOwnerCheck::NotApplicable | RootEntryOwnerCheck::CanonicalOwner => Vec::new(),
        RootEntryOwnerCheck::SuspiciousOwner(owner) => {
            let mut finding = base_finding(
                MOD_R012,
                FindingSeverity::Warning,
                path,
                "Root seam should not point visible entries at helper modules",
                format!(
                    "`{}` exposes `{}` from child module `{}` at line {}. Root entry surfaces should prefer canonical owner modules over helper/detail buckets.",
                    display_path(path),
                    owner.symbol_name,
                    owner.source_module,
                    owner.line_number
                ),
            );
            finding.why_it_matters = "When the root seam points coding agents at `internal`, `detail`, or `helpers` modules, the first hop is likely to land on a support bucket instead of the real owner seam. That makes the folder structure less trustworthy as a navigation guide.".to_string();
            finding.remediation = "Expose canonical entry symbols from the real owner module, and keep helper/detail modules behind that seam. If the helper module truly owns the concept, rename the module so the ownership is explicit.".to_string();
            finding.examples.good.push(
                "`pub(crate) use self::service::Service;` or `pub use self::parser::Parser;` from modules whose names match the feature ownership seam."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`pub use self::internal::FeatureState;` or `pub(crate) use self::helpers::Builder;` at the root seam."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                owner.line_number,
                owner.rendered_path,
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_hint_specificity(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_doc_hint::check_root_doc_hint(path, text) {
        RootDocHintCheck::NotApplicable | RootDocHintCheck::SpecificHint => Vec::new(),
        RootDocHintCheck::DocWithoutChildModuleName(metrics) => {
            let mut finding = base_finding(
                MOD_R014,
                FindingSeverity::Warning,
                path,
                "Doc-only root seam should name a child module",
                format!(
                    "`{}` relies on a root `//!` hint at line {} but that hint names none of the {} declared child modules. Coding agents still need one concrete leaf seam to open first.",
                    display_path(path),
                    metrics.line_number,
                    metrics.child_modules
                ),
            );
            finding.why_it_matters = "A generic root doc such as `feature seam` is better than nothing, but a doc-only seam still leaves Codex guessing which child module owns the first real implementation hop. Naming one child module keeps the directory layout actionable.".to_string();
            finding.remediation = "When the root seam does not expose a visible entry export, update the root `//!` doc to name at least one declared child module such as `service`, `parser`, or `runtime`. If a stable visible entry already exists, prefer that seam instead.".to_string();
            finding.examples.good.push(
                "`//! Service + runtime seam for the feature.` or `//! Start in `service`, then descend into `parser`.`"
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Feature seam for the demo.` without any visible entry export or named child module."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.line_number,
                metrics.doc_preview,
            ));
            vec![finding]
        }
    }
}

fn check_root_entry_focus(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_entry_focus::check_root_entry_focus(path, text) {
        RootEntryFocusCheck::NotApplicable | RootEntryFocusCheck::FocusedEntry => Vec::new(),
        RootEntryFocusCheck::UnfocusedEntry(metrics) => {
            let mut finding = base_finding(
                MOD_R015,
                FindingSeverity::Warning,
                path,
                "Root seam should identify a primary entry owner",
                format!(
                    "`{}` exposes visible entries from {} child modules at line {} but no source module clearly dominates and the root seam names no primary owner. Coding agents still need one first leaf seam.",
                    display_path(path),
                    metrics.source_modules,
                    metrics.first_export_line
                ),
            );
            finding.why_it_matters = "A root seam with several peer entry owners still reads like a flat list to Codex even when the export surface is small. One explicit primary owner keeps the first-hop path stable for local edits and deeper navigation.".to_string();
            finding.remediation = "Either keep one child module as the clear primary entry owner, or add a short root `//!` doc that names the primary module such as `service` before listing supporting modules like `parser` or `runtime`.".to_string();
            finding.examples.good.push(
                "`//! Start in service; parser and runtime support the seam.` plus visible entries from several child modules."
                    .to_string(),
            );
            finding.examples.bad.push(
                "A root seam that re-exports one visible entry from `service`, `parser`, and `runtime` without any hint about the primary owner."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.first_export_line,
                format!(
                    "source_modules={}, top_source_count={}",
                    metrics.source_modules, metrics.top_source_count
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_owner_alignment(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_doc_owner_alignment::check_root_doc_owner_alignment(path, text) {
        RootDocOwnerAlignmentCheck::NotApplicable | RootDocOwnerAlignmentCheck::Aligned => {
            Vec::new()
        }
        RootDocOwnerAlignmentCheck::Misaligned(metrics) => {
            let mut finding = base_finding(
                MOD_R016,
                FindingSeverity::Warning,
                path,
                "Root doc owner hint should align with visible entry surface",
                format!(
                    "`{}` names root-doc owner module(s) {} at line {}, but the visible entry surface only exports from {}. If the doc tells Codex where to start, the root seam should expose that same owner.",
                    display_path(path),
                    metrics.named_modules,
                    metrics.doc_line_number,
                    metrics.visible_modules
                ),
            );
            finding.why_it_matters = "When the root `//!` doc points coding agents at one child module but the visible entry surface exposes a different set of owners, the seam sends mixed signals. That increases the chance of opening the wrong leaf file first.".to_string();
            finding.remediation = "Either expose at least one visible entry from the child module named in the root doc, or update the root doc so it names the same owner modules that actually shape the visible entry seam.".to_string();
            finding.examples.good.push(
                "`//! Start in service.` plus `pub(crate) use self::service::Service;` keeps the doc and visible seam aligned."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Start in service.` while the visible entry seam only re-exports `parser` and `runtime` symbols."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "named_modules={}, visible_modules={}",
                    metrics.named_modules, metrics.visible_modules
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_owner_convergence(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_owner_convergence::check_root_owner_convergence(path, text) {
        RootOwnerConvergenceCheck::NotApplicable | RootOwnerConvergenceCheck::Converged => {
            Vec::new()
        }
        RootOwnerConvergenceCheck::Drifted(metrics) => {
            let mut finding = base_finding(
                MOD_R017,
                FindingSeverity::Warning,
                path,
                "Root seam should converge on one owner",
                format!(
                    "`{}` names {} in the root doc at line {}, but the dominant visible owner is `{}` with {} entry exports. Once the root seam points Codex at one owner, the visible seam should not drift toward another.",
                    display_path(path),
                    metrics.named_modules,
                    metrics.doc_line_number,
                    metrics.dominant_module,
                    metrics.dominant_count
                ),
            );
            finding.why_it_matters = "Partial alignment is still confusing for coding agents: the root doc may mention one owner, the visible seam may include it, but most visible entries still come from another child module. That makes the real first hop ambiguous during edits.".to_string();
            finding.remediation = "Either keep the doc-named owner as the dominant visible entry module, or update the root doc so it names the same module that actually dominates the visible seam. The root doc, visible entry surface, and primary owner should converge on one child module.".to_string();
            finding.examples.good.push(
                "`//! Start in service.` plus a visible seam where `service` contributes the main entry exports and `parser` stays secondary."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Start in service.` while visible entries include `service` once and `parser` twice, making `parser` the real dominant owner."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.first_export_line,
                format!(
                    "named_modules={}, dominant_module={}, dominant_count={}",
                    metrics.named_modules, metrics.dominant_module, metrics.dominant_count
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_entry_visibility(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_entry_visibility::check_root_entry_visibility(path, text, file_texts) {
        RootEntryVisibilityCheck::NotApplicable | RootEntryVisibilityCheck::Allowed => Vec::new(),
        RootEntryVisibilityCheck::Overexposed(metrics) => {
            let mut finding = base_finding(
                MOD_R018,
                FindingSeverity::Warning,
                path,
                "Internal root seam should prefer restricted entry visibility",
                format!(
                    "`{}` is declared from its parent as `{}` but still re-exports `{}` with plain `pub` at line {}. Internal root seams should avoid advertising a broader owner surface than the parent module actually has.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.symbol_name,
                    metrics.line_number
                ),
            );
            finding.why_it_matters = "Once a root seam is internal because its parent module is private or restricted, a plain `pub use` makes the local owner surface look more public than it really is. That gives coding agents the wrong impression about how wide the seam is meant to be.".to_string();
            finding.remediation = "Use `pub(crate)` or `pub(super)` for internal root entry re-exports, keep child mounts private (`mod child;` or private `#[path = ...] mod child;`), and only make the parent fully `pub mod ...` when the seam is intentionally part of the crate's public API.".to_string();
            finding.examples.good.push(
                "`mod feature;` in the parent file plus `pub(crate) use self::service::Service;` inside `feature.rs`."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`mod feature;` or `pub(crate) mod feature;` in the parent file plus `pub use self::service::Service;` inside the root seam."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.line_number,
                metrics.rendered_path,
            ));
            vec![finding]
        }
    }
}

fn check_root_entry_curation(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_entry_curation::check_root_entry_curation(path, text, file_texts) {
        RootEntryCurationCheck::NotApplicable | RootEntryCurationCheck::Curated => Vec::new(),
        RootEntryCurationCheck::InternalMultiOwnerSurface(metrics) => {
            let mut finding = base_finding(
                MOD_R019,
                FindingSeverity::Warning,
                path,
                "Internal root seam should keep one canonical visible owner",
                format!(
                    "`{}` is declared from its parent as `{}` but still exposes visible entries from {} child-owner modules ({}) starting at line {}. For internal seams, coding agents usually only need one canonical visible owner; sibling folders and a short root doc can guide the rest.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.source_modules,
                    metrics.visible_modules,
                    metrics.first_export_line
                ),
            );
            finding.why_it_matters = "Even with restricted visibility, a root seam that re-exports several child owners still flattens the feature boundary into a small peer list. For Codex-style navigation, one canonical visible owner keeps the first hop tighter while the folder layout continues to explain the secondary leaves.".to_string();
            finding.remediation = "Keep one child module as the canonical visible owner in the internal root seam, and let secondary modules stay behind private `mod` or private `#[path = ...] mod` mounts. If secondary navigation matters, mention those modules in a short root `//!` doc instead of re-exporting them all.".to_string();
            finding.examples.good.push(
                "`mod feature;` in the parent file plus `pub(crate) use self::service::Service;` in `feature.rs`, with parser/runtime described in a short root doc if needed."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`mod feature;` in the parent file plus restricted re-exports from `service`, `parser`, and `runtime` in the same internal root seam."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.first_export_line,
                format!(
                    "visible_modules={}, source_modules={}",
                    metrics.visible_modules, metrics.source_modules
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_curation(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_doc_curation::check_root_doc_curation(path, text, file_texts) {
        RootDocCurationCheck::NotApplicable | RootDocCurationCheck::FocusedDoc => Vec::new(),
        RootDocCurationCheck::InventoryStyleRootDoc(metrics) => {
            let mut finding = base_finding(
                MOD_R020,
                FindingSeverity::Warning,
                path,
                "Internal root doc should stay focused on the canonical owner",
                format!(
                    "`{}` is declared from its parent as `{}` and already exposes canonical visible owner `{}`, but the root `//!` doc at line {} still names all declared child modules ({}). Coding agents can read the folder tree directly; the root hint should keep the first hop small.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.visible_owner,
                    metrics.doc_line_number,
                    metrics.named_modules
                ),
            );
            finding.why_it_matters = "Once an internal root seam already exposes one canonical visible owner, a root doc that inventories every child module adds token noise without improving navigation. Codex usually benefits more from one primary owner hint than from a prose mirror of the directory tree.".to_string();
            finding.remediation = "Keep the root `//!` doc centered on the canonical visible owner module, and mention only the one or two secondary seams that truly need extra context. Let the sibling file layout carry the rest of the structure.".to_string();
            finding.examples.good.push(
                "`//! Start in service; parser stays leaf-owned.` keeps the first hop explicit without restating the whole folder."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! service owns entry, parser handles syntax, runtime executes requests.` when the root seam already exposes `service` as the canonical owner."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "visible_owner={}, named_modules={}",
                    metrics.visible_owner, metrics.named_modules
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_priority(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_doc_priority::check_root_doc_priority(path, text, file_texts) {
        RootDocPriorityCheck::NotApplicable | RootDocPriorityCheck::OwnerFirst => Vec::new(),
        RootDocPriorityCheck::SecondaryMentionedFirst(metrics) => {
            let mut finding = base_finding(
                MOD_R021,
                FindingSeverity::Warning,
                path,
                "Internal root doc should mention the canonical owner first",
                format!(
                    "`{}` is declared from its parent as `{}` and exposes canonical visible owner `{}`, but the root `//!` doc at line {} mentions secondary module `{}` before that owner. Coding agents usually treat the first named module as the first hop.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.visible_owner,
                    metrics.doc_line_number,
                    metrics.leading_module
                ),
            );
            finding.why_it_matters = "Even when the root doc and visible seam name the same modules, ordering still shapes navigation. If a secondary seam appears before the canonical visible owner, Codex can open the wrong leaf file first.".to_string();
            finding.remediation = "Rewrite the root `//!` hint so the canonical visible owner module appears before any secondary seams such as `parser` or `runtime`. Keep secondary context after the first-hop owner.".to_string();
            finding.examples.good.push(
                "`//! Start in service; parser stays leaf-owned.` keeps the primary seam first."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Parser handles syntax; start in service.` puts the secondary seam ahead of the canonical visible owner."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "visible_owner={}, leading_module={}",
                    metrics.visible_owner, metrics.leading_module
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_secondary_budget(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_doc_secondary_budget::check_root_doc_secondary_budget(path, text, file_texts) {
        RootDocSecondaryBudgetCheck::NotApplicable | RootDocSecondaryBudgetCheck::WithinBudget => {
            Vec::new()
        }
        RootDocSecondaryBudgetCheck::TooManySecondaryMentions(metrics) => {
            let mut finding = base_finding(
                MOD_R022,
                FindingSeverity::Warning,
                path,
                "Internal root doc should limit secondary seam mentions",
                format!(
                    "`{}` is declared from its parent as `{}` and already exposes canonical visible owner `{}`, but the root `//!` doc at line {} still names {} secondary seams ({}). Coding agents can read the sibling tree directly, so the root hint should stay short.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.visible_owner,
                    metrics.doc_line_number,
                    metrics.secondary_mentions,
                    metrics.secondary_modules
                ),
            );
            finding.why_it_matters = "Even when ordering is correct, naming several secondary seams in the root doc adds token noise and encourages coding agents to treat the root hint like a mini README. One canonical owner plus at most one secondary seam is usually enough.".to_string();
            finding.remediation = "Keep the root `//!` doc centered on the canonical visible owner and at most one secondary seam that genuinely needs extra context. Let the sibling folder structure carry the rest.".to_string();
            finding.examples.good.push(
                "`//! Start in service; parser handles syntax.` keeps the root hint compact."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Start in service; parser handles syntax; runtime executes; storage persists.` turns the root hint into a directory summary."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "visible_owner={}, secondary_modules={}, secondary_mentions={}",
                    metrics.visible_owner, metrics.secondary_modules, metrics.secondary_mentions
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_secondary_owner(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_doc_secondary_owner::check_root_doc_secondary_owner(path, text, file_texts) {
        RootDocSecondaryOwnerCheck::NotApplicable
        | RootDocSecondaryOwnerCheck::CanonicalSecondary => Vec::new(),
        RootDocSecondaryOwnerCheck::SuspiciousSecondary(metrics) => {
            let mut finding = base_finding(
                MOD_R023,
                FindingSeverity::Warning,
                path,
                "Internal root doc should avoid helper-bucket secondary seams",
                format!(
                    "`{}` is declared from its parent as `{}` and already exposes canonical visible owner `{}`, but the root `//!` doc at line {} still names helper/detail bucket `{}` as the secondary seam. If Codex needs one extra hop, it should still be a canonical feature seam.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.visible_owner,
                    metrics.doc_line_number,
                    metrics.secondary_module
                ),
            );
            finding.why_it_matters = "A short root doc is only useful if the extra secondary seam still points at a real feature boundary. Naming `internal`, `detail`, or `helper` in the root hint can steer coding agents into support buckets instead of the real leaf owner path.".to_string();
            finding.remediation = "Keep the root `//!` hint on the canonical visible owner and, if one extra seam is still needed, name a canonical feature seam such as `parser`, `runtime`, or `service` instead of `internal` or `detail` buckets.".to_string();
            finding.examples.good.push(
                "`//! Start in service; parser handles syntax.` keeps the secondary seam meaningful."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`//! Start in service; internal handles glue.` points the secondary hint at a support bucket."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "visible_owner={}, secondary_module={}",
                    metrics.visible_owner, metrics.secondary_module
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_doc_owner_name(
    path: &Path,
    text: &str,
    file_texts: &BTreeMap<PathBuf, String>,
) -> Vec<ContractFinding> {
    match root_doc_owner_name::check_root_doc_owner_name(path, text, file_texts) {
        RootDocOwnerNameCheck::NotApplicable | RootDocOwnerNameCheck::CanonicalOwnerName => {
            Vec::new()
        }
        RootDocOwnerNameCheck::DriftedOwnerName(metrics) => {
            let mut finding = base_finding(
                MOD_R024,
                FindingSeverity::Warning,
                path,
                "Internal root doc should name the canonical owner module directly",
                format!(
                    "`{}` is declared from its parent as `{}` and already exposes canonical visible owner `{}`, but the root `//!` doc at line {} hints the first hop as `{}` instead of naming the real child module. When the root hint uses inline-code owner wording, Codex should see the actual owner path.",
                    display_path(path),
                    metrics.parent_visibility,
                    metrics.visible_owner,
                    metrics.doc_line_number,
                    metrics.hinted_owner
                ),
            );
            finding.why_it_matters = "A short root hint is most useful when its owner wording matches the real child module path that owns the feature. Alias or marketing-style owner names force coding agents to translate the hint back into the folder tree before they can open the right file.".to_string();
            finding.remediation = "When an internal root doc points to the canonical visible owner, name that owner with the real child module path such as `service` or `parser`. Avoid replacing the module name with type aliases or marketing labels in the root `//!` hint.".to_string();
            finding.examples.good.push(
                "`//! Start in `service`.` names the real owner module directly.".to_string(),
            );
            finding.examples.bad.push(
                "`//! Start in `FeatureService`.` makes Codex translate an alias back to the real `service` module."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                metrics.doc_line_number,
                format!(
                    "visible_owner={}, hinted_owner={}",
                    metrics.visible_owner, metrics.hinted_owner
                ),
            ));
            vec![finding]
        }
    }
}

fn check_root_child_visibility(path: &Path, text: &str) -> Vec<ContractFinding> {
    match root_child_visibility::check_root_child_visibility(path, text) {
        RootChildVisibilityCheck::NotApplicable | RootChildVisibilityCheck::PrivateChildrenOnly => {
            Vec::new()
        }
        RootChildVisibilityCheck::VisibleChildModule(item) => {
            let mut finding = base_finding(
                MOD_R013,
                FindingSeverity::Warning,
                path,
                "Root facade should keep child modules private",
                format!(
                    "`{}` declares child module `{}` with `{}` visibility at line {}. Root seams should expose canonical entry exports or a root hint instead of making child modules the visible boundary.",
                    display_path(path),
                    item.module_name,
                    item.visibility,
                    item.line_number
                ),
            );
            finding.why_it_matters = "When a folder-root seam publishes `pub mod child;`, coding agents learn the child module path as the contract surface and bypass the curated facade. Keeping child modules private preserves one stable seam at the root.".to_string();
            finding.remediation = format!(
                "Replace `{}` with private `mod {};` or private `#[path = \"../{}.rs\"] mod {};` when the owner file lives elsewhere, then expose only the intended entry symbols with selective `pub(crate) use` or `pub use` re-exports. If no visible export is needed, keep the mount private and add a short root `//!` hint instead.",
                format!("{} mod {};", item.visibility, item.module_name),
                item.module_name,
                item.module_name,
                item.module_name
            );
            finding.examples.good.push(
                "`mod service;` or private `#[path = \"../service.rs\"] mod service;` plus `pub(crate) use self::service::Service;` keeps the folder-root seam curated."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`pub mod service;` or `pub(crate) mod parser;` turns a child module declaration into the visible seam."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                item.line_number,
                format!("{} mod {};", item.visibility, item.module_name),
            ));
            vec![finding]
        }
    }
}

fn check_relative_import_clarity(path: &Path, text: &str) -> Vec<ContractFinding> {
    match relative_import::check_relative_import_clarity(text) {
        RelativeImportCheck::Clear => Vec::new(),
        RelativeImportCheck::MultiHopRelativeImport(import) => {
            let mut finding = base_finding(
                MOD_R009,
                FindingSeverity::Warning,
                path,
                "Prefer `crate::` over multi-hop relative imports",
                format!(
                    "`{}` uses `{}` at line {}. Prefer a `crate::...` path over `super::super::...` style imports.",
                    display_path(path),
                    import.rendered_path,
                    import.line_number
                ),
            );
            finding.why_it_matters = "Multi-hop relative imports are harder for both humans and coding agents to resolve quickly because the meaning depends on the current module depth. `crate::` paths keep the ownership seam explicit.".to_string();
            finding.remediation = "Replace `super::super::...` style imports with `crate::...` when the target lives in the same crate and the absolute crate path is stable.".to_string();
            finding.examples.good.push(
                "`use crate::contracts::model::ContractFinding;` keeps the target clear regardless of local nesting."
                    .to_string(),
            );
            finding.examples.bad.push(
                "`use super::super::model::ContractFinding;` forces readers and coding agents to count parent hops."
                    .to_string(),
            );
            finding.evidence.push(FindingEvidenceEntry::source_span(
                path,
                import.line_number,
                import.rendered_path,
            ));
            vec![finding]
        }
    }
}

fn check_visibility_boundary(path: &Path, text: &str) -> Vec<ContractFinding> {
    let file_name = path.file_name().and_then(|name| name.to_str());
    if matches!(file_name, Some("lib.rs" | "main.rs" | "mod.rs")) {
        return Vec::new();
    }
    if path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some("src")
    {
        return Vec::new();
    }

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !is_broad_public_item(trimmed) {
            continue;
        }

        let mut finding = base_finding(
            MOD_R002,
            FindingSeverity::Warning,
            path,
            "Public visibility may be broader than required",
            format!(
                "Public item at line {} in `{}` may need narrower visibility.",
                index + 1,
                display_path(path)
            ),
        );
        finding.why_it_matters = "Overly broad visibility weakens module boundaries and increases accidental API surface.".to_string();
        finding.remediation = "Use `pub(crate)` (or narrower) for internal items unless the symbol is intentionally part of the external crate API.".to_string();
        finding
            .examples
            .good
            .push("`pub(crate) struct InternalState` for crate-internal wiring.".to_string());
        finding
            .examples
            .bad
            .push("`pub struct` used only by sibling modules inside the same crate.".to_string());
        finding.evidence.push(FindingEvidenceEntry::source_span(
            path,
            index + 1,
            line.trim().to_string(),
        ));
        return vec![finding];
    }

    Vec::new()
}

fn check_public_result_error_docs(path: &Path, text: &str) -> Vec<ContractFinding> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !is_public_result_signature(trimmed) {
            continue;
        }
        if has_errors_doc(&lines, index) {
            continue;
        }

        let mut finding = base_finding(
            MOD_R003,
            FindingSeverity::Error,
            path,
            "Public Result API is missing `# Errors` docs",
            format!(
                "Public function at line {} in `{}` returns `Result` without `# Errors` documentation.",
                index + 1,
                display_path(path)
            ),
        );
        finding.why_it_matters =
            "Error contracts are part of public API semantics and should be documented explicitly."
                .to_string();
        finding.remediation = "Add a doc-comment section with `# Errors` that describes failure conditions for this API.".to_string();
        finding
            .examples
            .good
            .push("`/// # Errors` followed by concrete failure scenarios.".to_string());
        finding
            .examples
            .bad
            .push("Public `Result` function with no error contract docs.".to_string());
        finding.evidence.push(FindingEvidenceEntry::source_span(
            path,
            index + 1,
            line.trim().to_string(),
        ));
        findings.push(finding);
    }

    findings
}

fn has_errors_doc(lines: &[&str], signature_index: usize) -> bool {
    let mut saw_doc = false;
    for cursor in (0..signature_index).rev() {
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            if saw_doc {
                break;
            }
            continue;
        }
        if trimmed.starts_with("#[") {
            continue;
        }
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            saw_doc = true;
            if trimmed.contains("# Errors") {
                return true;
            }
            continue;
        }
        break;
    }
    false
}

fn is_broad_public_item(line: &str) -> bool {
    line.starts_with("pub ")
        && !line.starts_with("pub(crate)")
        && !line.starts_with("pub(super)")
        && !line.starts_with("pub(in ")
        && !line.starts_with("pub use ")
        && (line.starts_with("pub fn ")
            || line.starts_with("pub async fn ")
            || line.starts_with("pub struct ")
            || line.starts_with("pub enum ")
            || line.starts_with("pub trait ")
            || line.starts_with("pub type ")
            || line.starts_with("pub const ")
            || line.starts_with("pub static "))
}

fn is_public_result_signature(line: &str) -> bool {
    let is_public_function = line.starts_with("pub fn ") || line.starts_with("pub async fn ");
    let returns_result = line.contains("->") && line.contains("Result<");
    is_public_function && returns_result
}

fn base_finding(
    rule_id: &str,
    severity: FindingSeverity,
    path: &Path,
    title: impl Into<String>,
    summary: impl Into<String>,
) -> ContractFinding {
    let mut finding = ContractFinding::new(
        rule_id,
        PACK_ID,
        severity,
        FindingMode::Deterministic,
        title,
        summary,
    );
    finding
        .labels
        .insert("domain".to_string(), "modularity".to_string());
    finding
        .labels
        .insert("path".to_string(), display_path(path));
    finding
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

struct FindingEvidenceEntry;

impl FindingEvidenceEntry {
    fn source_span(path: &Path, line_number: usize, message: String) -> FindingEvidence {
        FindingEvidence {
            kind: EvidenceKind::SourceSpan,
            path: Some(path.to_path_buf()),
            locator: Some(format!("line:{line_number}")),
            message,
        }
    }
}
