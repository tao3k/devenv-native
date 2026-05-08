"""Cargo feature selection helpers for benchmark probes."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import argparse


def cargo_features_with_pdf_render(features: str) -> str:
    return cargo_features_with_pdf_feature(features, "document-extract-pdf-render")


def cargo_features_for_studio_pdf_render_audit(features: str) -> str:
    return cargo_features_with_pdf_feature(
        cargo_features_without_package_foreign_entries(
            features,
            {"document-extract-attachment-audit"},
        ),
        "document-extract-pdf-render",
    )


def cargo_features_with_pdf_source_range(features: str) -> str:
    return cargo_features_with_pdf_feature(
        features, "document-extract-pdf-source-range"
    )


def cargo_features_for_flight_mode(features: str, flight_mode: str) -> str:
    if flight_mode == "hybrid-page-ocr":
        return cargo_features_with_pdf_source_range(features)
    return features


def cargo_features_for_provider_mode(features: str, args: argparse.Namespace) -> str:
    flight_mode = getattr(args, "flight_mode", "sync")
    if flight_mode != "hybrid-page-ocr":
        return features
    profile_planner = str(getattr(args, "rust_pdf_ocr_profile_planner", "")).replace(
        "_", "-"
    )
    if profile_planner in {
        "hosted-vlm-all",
        "hosted-vlm-risk-window",
        "hosted-vlm-risk-window-backend-text",
    }:
        return cargo_features_with_pdf_render(features)
    failed_page_recovery = str(
        getattr(args, "rust_pdf_failed_page_recovery", "")
    ).replace("_", "-")
    if failed_page_recovery == "hosted-vlm-page":
        return cargo_features_with_pdf_render(features)
    selection = normalize_render_selection(
        getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
    )
    if selection == "region_shards":
        return cargo_features_with_pdf_render(features)
    return cargo_features_with_pdf_source_range(features)


def cargo_features_with_pdf_feature(features: str, feature: str) -> str:
    parts = [
        part.strip()
        for chunk in features.split(",")
        for part in chunk.split()
        if part.strip()
    ]
    if feature not in parts:
        parts.append(feature)
    if "performance" not in parts:
        parts.insert(0, "performance")
    return ",".join(parts)


def cargo_features_without_package_foreign_entries(
    features: str,
    package_foreign_features: set[str],
) -> str:
    parts = [
        part.strip()
        for chunk in features.split(",")
        for part in chunk.split()
        if part.strip()
    ]
    return ",".join(part for part in parts if part not in package_foreign_features)


def normalize_render_selection(selection: str) -> str:
    return selection.strip().replace("-", "_")
