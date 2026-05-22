use std::collections::BTreeSet;
use std::path::Path;

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_BACKEND_TEXT_PROFILE, PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PdfOcrShardInput,
};
use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageClassification, PdfSourcePageProfile, classify_pdf_source_pages,
    pdf_source_page_is_backend_text_topup_profile, pdf_source_page_is_fast_profile_risk,
    source_pdf_page_profiles_cached,
};

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER";
pub(crate) const DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP";
const PDF_OCR_FAST_TEXT_ENGINE: &str = "docling-fast-text-ocr";
const PDF_OCR_BACKEND_TEXT_ENGINE: &str = "docling-backend-text-ocr";
const PDF_OCR_HOSTED_VLM_DIRECT_ENGINE: &str = "hosted-vlm-direct-ocr";
pub(crate) const PDF_OCR_HOSTED_VLM_TOPUP_ENGINE: &str = "hosted-vlm-topup-ocr";
const FAST_ALL_MODE: &str = "fast-all";
const FAST_RISK_WINDOW_MODE: &str = "fast-risk-window";
const HOSTED_VLM_ALL_MODE: &str = "hosted-vlm-all";
const HOSTED_VLM_RISK_WINDOW_MODE: &str = "hosted-vlm-risk-window";
const HOSTED_VLM_RISK_WINDOW_BACKEND_TEXT_MODE: &str = "hosted-vlm-risk-window-backend-text";
const DOCLING_STRUCTURE_RECOVERY_MODE: &str = "docling-structure-recovery";
const PDF_OCR_DEFAULT_ENGINE: &str = "docling-compatible-ocr";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcrProfilePlanner {
    Disabled,
    FastAll,
    FastRiskWindow,
    HostedVlmAll,
    HostedVlmRiskWindow,
    HostedVlmRiskWindowBackendText,
    DoclingStructureRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfBackendTextTopup {
    Profile,
    Disabled,
    HostedVlm,
}

pub(crate) fn apply_hybrid_page_ocr_profile_plan(
    inputs: Vec<PdfOcrShardInput>,
) -> Vec<PdfOcrShardInput> {
    match hybrid_page_ocr_profile_planner() {
        HybridPdfOcrProfilePlanner::Disabled => inputs,
        HybridPdfOcrProfilePlanner::FastAll => {
            if eligible_source_path(inputs.as_slice()).is_some() {
                apply_candidate_profile_plan(
                    inputs,
                    &BTreeSet::new(),
                    PDF_OCR_FAST_TEXT_PROFILE,
                    PDF_OCR_FAST_TEXT_ENGINE,
                )
            } else {
                inputs
            }
        }
        HybridPdfOcrProfilePlanner::FastRiskWindow => {
            let Some(source_path) = eligible_source_path(inputs.as_slice()) else {
                return inputs;
            };
            let profiles = match source_pdf_page_profiles_cached(Path::new(source_path.as_str())) {
                Ok(profiles) => profiles,
                Err(error) => {
                    log::debug!("hybrid PDF OCR profile planner skipped source profile: {error}");
                    return inputs;
                }
            };
            apply_hybrid_page_ocr_profile_plan_for_profiles(inputs, profiles.as_slice())
        }
        HybridPdfOcrProfilePlanner::HostedVlmAll => {
            if eligible_source_path(inputs.as_slice()).is_some() {
                apply_candidate_profile_plan(
                    inputs,
                    &BTreeSet::new(),
                    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
                    PDF_OCR_HOSTED_VLM_DIRECT_ENGINE,
                )
            } else {
                inputs
            }
        }
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindow => {
            let Some(source_path) = eligible_source_path(inputs.as_slice()) else {
                return inputs;
            };
            let profiles = match source_pdf_page_profiles_cached(Path::new(source_path.as_str())) {
                Ok(profiles) => profiles,
                Err(error) => {
                    log::debug!("hybrid PDF OCR profile planner skipped source profile: {error}");
                    return inputs;
                }
            };
            apply_hybrid_page_hosted_vlm_profile_plan_for_profiles(inputs, profiles.as_slice())
        }
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindowBackendText => {
            let Some(source_path) = eligible_source_path(inputs.as_slice()) else {
                return inputs;
            };
            let profiles = match source_pdf_page_profiles_cached(Path::new(source_path.as_str())) {
                Ok(profiles) => profiles,
                Err(error) => {
                    log::debug!("hybrid PDF OCR profile planner skipped source profile: {error}");
                    return inputs;
                }
            };
            apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles(
                inputs,
                profiles.as_slice(),
            )
        }
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery => {
            let Some(source_path) = eligible_source_path(inputs.as_slice()) else {
                return inputs;
            };
            let profiles = match source_pdf_page_profiles_cached(Path::new(source_path.as_str())) {
                Ok(profiles) => profiles,
                Err(error) => {
                    log::debug!("hybrid PDF OCR profile planner skipped source profile: {error}");
                    return inputs;
                }
            };
            apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles(
                inputs,
                profiles.as_slice(),
            )
        }
    }
}

pub(crate) fn hybrid_page_ocr_profile_planner() -> HybridPdfOcrProfilePlanner {
    hybrid_page_ocr_profile_planner_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn hybrid_pdf_backend_text_topup_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfBackendTextTopup {
    match lookup(DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        "disabled" => HybridPdfBackendTextTopup::Disabled,
        "hosted-vlm" => HybridPdfBackendTextTopup::HostedVlm,
        _ => HybridPdfBackendTextTopup::Profile,
    }
}

impl HybridPdfOcrProfilePlanner {
    pub(crate) fn requires_rendered_page_images(self) -> bool {
        matches!(self, Self::HostedVlmAll)
    }
}

pub(crate) fn hybrid_page_ocr_profile_planner_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcrProfilePlanner {
    match lookup(DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        FAST_ALL_MODE => HybridPdfOcrProfilePlanner::FastAll,
        FAST_RISK_WINDOW_MODE => HybridPdfOcrProfilePlanner::FastRiskWindow,
        HOSTED_VLM_ALL_MODE => HybridPdfOcrProfilePlanner::HostedVlmAll,
        HOSTED_VLM_RISK_WINDOW_MODE => HybridPdfOcrProfilePlanner::HostedVlmRiskWindow,
        HOSTED_VLM_RISK_WINDOW_BACKEND_TEXT_MODE => {
            HybridPdfOcrProfilePlanner::HostedVlmRiskWindowBackendText
        }
        DOCLING_STRUCTURE_RECOVERY_MODE => HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
        _ => HybridPdfOcrProfilePlanner::Disabled,
    }
}

pub(crate) fn apply_hybrid_page_ocr_profile_plan_for_profiles(
    inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfOcrShardInput> {
    if eligible_source_path(inputs.as_slice()).is_none() {
        return inputs;
    }
    let profile_pages = profiles
        .iter()
        .map(|profile| profile.page_index)
        .collect::<BTreeSet<_>>();
    if inputs
        .iter()
        .any(|input| !profile_pages.contains(&input.page_index))
    {
        return inputs;
    }

    let accurate_pages = accurate_recovery_pages(profiles);
    if accurate_pages.is_empty() || accurate_pages.len() >= inputs.len() {
        return inputs;
    }
    apply_candidate_profile_plan(
        inputs,
        &accurate_pages,
        PDF_OCR_FAST_TEXT_PROFILE,
        PDF_OCR_FAST_TEXT_ENGINE,
    )
}

pub(crate) fn apply_hybrid_page_hosted_vlm_profile_plan_for_profiles(
    inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfOcrShardInput> {
    if eligible_source_path(inputs.as_slice()).is_none() {
        return inputs;
    }
    let profile_pages = profiles
        .iter()
        .map(|profile| profile.page_index)
        .collect::<BTreeSet<_>>();
    if inputs
        .iter()
        .any(|input| !profile_pages.contains(&input.page_index))
    {
        return inputs;
    }

    let recovery_pages = accurate_recovery_pages(profiles);
    apply_hosted_vlm_recovery_profile_plan(
        inputs,
        &recovery_pages,
        PDF_OCR_FAST_TEXT_PROFILE,
        PDF_OCR_FAST_TEXT_ENGINE,
    )
}

pub(crate) fn apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles(
    inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfOcrShardInput> {
    apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup(
        inputs,
        profiles,
        &|key| std::env::var(key).ok(),
    )
}

pub(crate) fn apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles(
    mut inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfOcrShardInput> {
    if eligible_source_path(inputs.as_slice()).is_none() {
        return inputs;
    }
    let classifications = classify_pdf_source_pages(profiles);
    let profile_pages = classifications
        .iter()
        .map(|classification| classification.page_index)
        .collect::<BTreeSet<_>>();
    if inputs
        .iter()
        .any(|input| !profile_pages.contains(&input.page_index))
    {
        return inputs;
    }

    let mut structure_count = 0usize;
    let mut patch_count = 0usize;
    let mut fast_text_count = 0usize;
    let mut backend_text_count = 0usize;
    let mut default_count = 0usize;

    for input in &mut inputs {
        let Some(classification) = classification_for_page(&classifications, input.page_index)
        else {
            continue;
        };
        let Some(profile) = profiles
            .iter()
            .find(|profile| profile.page_index == input.page_index)
        else {
            continue;
        };
        if classification.ocr_patch_candidate {
            input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_HOSTED_VLM_DIRECT_ENGINE.to_string();
            patch_count = patch_count.saturating_add(1);
        } else if classification.structure_authority_required {
            input.ocr_profile = PDF_OCR_DEFAULT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_DEFAULT_ENGINE.to_string();
            structure_count = structure_count.saturating_add(1);
        } else if classification.text_shortcut_eligible {
            if pdf_source_page_is_backend_text_topup_profile(profile) {
                input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
                input.ocr_engine = PDF_OCR_FAST_TEXT_ENGINE.to_string();
                fast_text_count = fast_text_count.saturating_add(1);
            } else {
                input.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
                input.ocr_engine = PDF_OCR_BACKEND_TEXT_ENGINE.to_string();
                backend_text_count = backend_text_count.saturating_add(1);
            }
        } else {
            input.ocr_profile = PDF_OCR_DEFAULT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_DEFAULT_ENGINE.to_string();
            default_count = default_count.saturating_add(1);
        }
    }
    log::info!(
        "hybrid PDF OCR Docling structure recovery planner selected {structure_count} structure-authority pages, {patch_count} hosted patch pages, {fast_text_count} fast-text pages, {backend_text_count} backend-text pages, and {default_count} default Docling pages"
    );
    inputs
}

pub(crate) fn apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup(
    mut inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<PdfOcrShardInput> {
    if eligible_source_path(inputs.as_slice()).is_none() {
        return inputs;
    }
    let profile_pages = profiles
        .iter()
        .map(|profile| profile.page_index)
        .collect::<BTreeSet<_>>();
    if inputs
        .iter()
        .any(|input| !profile_pages.contains(&input.page_index))
    {
        return inputs;
    }

    let recovery_pages = accurate_recovery_pages(profiles);
    let topup_mode = hybrid_pdf_backend_text_topup_with_lookup(lookup);
    let topup_pages = match topup_mode {
        HybridPdfBackendTextTopup::Disabled => BTreeSet::new(),
        HybridPdfBackendTextTopup::Profile | HybridPdfBackendTextTopup::HostedVlm => {
            backend_text_topup_pages(profiles)
        }
    };
    let mut hosted_count = 0usize;
    let mut fast_topup_count = 0usize;
    let mut hosted_topup_count = 0usize;
    let mut backend_count = 0usize;
    for input in &mut inputs {
        if recovery_pages.contains(&input.page_index) {
            input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_HOSTED_VLM_DIRECT_ENGINE.to_string();
            hosted_count = hosted_count.saturating_add(1);
        } else if topup_pages.contains(&input.page_index) {
            match topup_mode {
                HybridPdfBackendTextTopup::Profile => {
                    input.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
                    input.ocr_engine = PDF_OCR_FAST_TEXT_ENGINE.to_string();
                    fast_topup_count = fast_topup_count.saturating_add(1);
                }
                HybridPdfBackendTextTopup::Disabled => {
                    input.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
                    input.ocr_engine = PDF_OCR_BACKEND_TEXT_ENGINE.to_string();
                    backend_count = backend_count.saturating_add(1);
                }
                HybridPdfBackendTextTopup::HostedVlm => {
                    input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
                    input.ocr_engine = PDF_OCR_HOSTED_VLM_TOPUP_ENGINE.to_string();
                    hosted_topup_count = hosted_topup_count.saturating_add(1);
                }
            }
        } else {
            input.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_BACKEND_TEXT_ENGINE.to_string();
            backend_count = backend_count.saturating_add(1);
        }
    }
    log::info!(
        "hybrid PDF OCR profile planner selected {hosted_count} hosted VLM recovery pages, {hosted_topup_count} hosted VLM top-up pages, {fast_topup_count} fast-text top-up pages, and {backend_count} backend-text pages"
    );
    inputs
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn is_hosted_vlm_topup_page(input: &PdfOcrShardInput) -> bool {
    input.shard_type == "page"
        && input.ocr_profile == PDF_OCR_HOSTED_VLM_DIRECT_PROFILE
        && input.ocr_engine == PDF_OCR_HOSTED_VLM_TOPUP_ENGINE
}

fn apply_candidate_profile_plan(
    mut inputs: Vec<PdfOcrShardInput>,
    accurate_pages: &BTreeSet<u32>,
    candidate_profile: &str,
    candidate_engine: &str,
) -> Vec<PdfOcrShardInput> {
    let mut candidate_count = 0usize;
    for input in &mut inputs {
        if accurate_pages.contains(&input.page_index) {
            input.ocr_profile = PDF_OCR_DEFAULT_PROFILE.to_string();
            input.ocr_engine = "docling-compatible-ocr".to_string();
        } else {
            input.ocr_profile = candidate_profile.to_string();
            input.ocr_engine = candidate_engine.to_string();
            candidate_count = candidate_count.saturating_add(1);
        }
    }
    log::info!(
        "hybrid PDF OCR profile planner selected {candidate_count} `{candidate_profile}` pages and {} accurate pages",
        inputs.len().saturating_sub(candidate_count)
    );
    inputs
}

fn apply_hosted_vlm_recovery_profile_plan(
    mut inputs: Vec<PdfOcrShardInput>,
    recovery_pages: &BTreeSet<u32>,
    default_profile: &str,
    default_engine: &str,
) -> Vec<PdfOcrShardInput> {
    let mut recovery_count = 0usize;
    for input in &mut inputs {
        if recovery_pages.contains(&input.page_index) {
            input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
            input.ocr_engine = PDF_OCR_HOSTED_VLM_DIRECT_ENGINE.to_string();
            recovery_count = recovery_count.saturating_add(1);
        } else {
            input.ocr_profile = default_profile.to_string();
            input.ocr_engine = default_engine.to_string();
        }
    }
    log::info!(
        "hybrid PDF OCR profile planner selected {recovery_count} hosted VLM recovery pages and {} `{default_profile}` pages",
        inputs.len().saturating_sub(recovery_count)
    );
    inputs
}

fn eligible_source_path(inputs: &[PdfOcrShardInput]) -> Option<String> {
    let first = inputs.first()?;
    if first.ocr_profile != PDF_OCR_DEFAULT_PROFILE {
        return None;
    }
    if inputs.iter().any(|input| {
        input.source_path != first.source_path
            || input.shard_type != "page"
            || input.ocr_profile != PDF_OCR_DEFAULT_PROFILE
    }) {
        return None;
    }
    Some(first.source_path.clone())
}

fn accurate_recovery_pages(profiles: &[PdfSourcePageProfile]) -> BTreeSet<u32> {
    let existing_pages = profiles
        .iter()
        .map(|profile| profile.page_index)
        .collect::<BTreeSet<_>>();
    let mut pages = BTreeSet::new();
    for profile in profiles {
        if !is_fast_profile_risk(profile) {
            continue;
        }
        for page_index in
            profile.page_index.saturating_sub(1)..=profile.page_index.saturating_add(1)
        {
            if existing_pages.contains(&page_index) {
                pages.insert(page_index);
            }
        }
    }
    pages
}

fn is_fast_profile_risk(profile: &PdfSourcePageProfile) -> bool {
    pdf_source_page_is_fast_profile_risk(profile)
}

fn backend_text_topup_pages(profiles: &[PdfSourcePageProfile]) -> BTreeSet<u32> {
    profiles
        .iter()
        .filter(|profile| is_backend_text_topup_profile(profile))
        .map(|profile| profile.page_index)
        .collect()
}

fn is_backend_text_topup_profile(profile: &PdfSourcePageProfile) -> bool {
    pdf_source_page_is_backend_text_topup_profile(profile)
}

fn classification_for_page(
    classifications: &[PdfSourcePageClassification],
    page_index: u32,
) -> Option<&PdfSourcePageClassification> {
    classifications
        .iter()
        .find(|classification| classification.page_index == page_index)
}

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/hybrid/profile.rs"]
mod tests;
