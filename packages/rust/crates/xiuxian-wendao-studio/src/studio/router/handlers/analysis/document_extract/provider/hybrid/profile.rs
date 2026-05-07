use std::collections::BTreeSet;
use std::path::Path;

use xiuxian_wendao_attachments::pdf::ocr::{
    PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    PdfOcrShardInput,
};
use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageProfile, source_pdf_page_profiles_cached,
};

pub(crate) const DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER";
const PDF_OCR_FAST_TEXT_ENGINE: &str = "docling-fast-text-ocr";
const PDF_OCR_HOSTED_VLM_DIRECT_ENGINE: &str = "hosted-vlm-direct-ocr";
const FAST_ALL_MODE: &str = "fast-all";
const FAST_RISK_WINDOW_MODE: &str = "fast-risk-window";
const OCR2_ALL_MODE: &str = "ocr2-all";
const OCR2_RISK_WINDOW_MODE: &str = "ocr2-risk-window";
const HOSTED_VLM_ALL_MODE: &str = "hosted-vlm-all";
const HOSTED_VLM_RISK_WINDOW_MODE: &str = "hosted-vlm-risk-window";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcrProfilePlanner {
    Disabled,
    FastAll,
    FastRiskWindow,
    HostedVlmAll,
    HostedVlmRiskWindow,
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
    }
}

pub(crate) fn hybrid_page_ocr_profile_planner() -> HybridPdfOcrProfilePlanner {
    hybrid_page_ocr_profile_planner_with_lookup(&|key| std::env::var(key).ok())
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
        OCR2_ALL_MODE | HOSTED_VLM_ALL_MODE => HybridPdfOcrProfilePlanner::HostedVlmAll,
        OCR2_RISK_WINDOW_MODE | HOSTED_VLM_RISK_WINDOW_MODE => {
            HybridPdfOcrProfilePlanner::HostedVlmRiskWindow
        }
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

#[cfg(test)]
pub(crate) fn apply_hybrid_page_ocr2_profile_plan_for_profiles(
    inputs: Vec<PdfOcrShardInput>,
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfOcrShardInput> {
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles(inputs, profiles)
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
    let compact_table_grid = (1..=8).contains(&profile.rectangle_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 120;
    let dense_table_path_band = (64..=120).contains(&profile.path_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 150;
    compact_table_grid || dense_table_path_band
}
