//! PDFium-backed document rendering and source-range manifest generation.

#[cfg(feature = "pdf-render")]
use image::DynamicImage;
#[cfg(feature = "pdf-render")]
use num_traits::ToPrimitive;
#[cfg(feature = "pdf-render")]
use pdfium_render::prelude::{
    PdfBitmapFormat, PdfDocument, PdfPage, PdfRenderConfig, Pdfium, PdfiumError,
};
#[cfg(feature = "pdf-render")]
use std::{fs, path::Path, sync::Mutex};

use crate::pdf::source_range::{
    source_page_range_all_page_indices, source_page_range_validate_page_index,
};

use super::identity::{checked_len_u32, sha256_hex};
#[cfg(feature = "pdf-render")]
use super::identity::{checked_pixels_i32, rotation_to_degrees, shard_element_id};
#[cfg(feature = "pdf-render")]
use super::manifest::{build_region_shard_manifest, region_pixel_box_for_crop};
use super::manifest::{build_shard_manifest, render_dimensions_for_box};
use super::report::{RenderShardContext, ReportParts};
use super::types::{
    PdfPageBox, PdfPageShardManifest, PdfPageShardManifestInput, RenderedRasterIdentity,
};
#[cfg(feature = "pdf-render")]
use super::types::{
    PdfPagePixelBox, PdfPageRegion, PdfPageRegionRenderRequest, PdfPageRegionShardManifestInput,
    PdfPageRenderProfile,
};

/// Environment variable that points to a dynamically loaded `PDFium` library.
#[cfg(feature = "pdf-render")]
pub const PDFIUM_LIBRARY_PATH_ENV: &str = "WENDAO_PDFIUM_LIBRARY_PATH";
#[cfg(feature = "pdf-render")]
const PDF_REGION_RENDER_MODE_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_PDF_REGION_RENDER_MODE";
#[cfg(feature = "pdf-render")]
const PDF_REGION_RENDER_MODE_DIRECT_CROP: &str = "direct-crop";
#[cfg(feature = "pdf-render")]
static PDFIUM_BIND_LOCK: Mutex<()> = Mutex::new(());

#[cfg(feature = "pdf-render")]
pub(super) fn bind_pdfium() -> Result<Pdfium, String> {
    let _guard = PDFIUM_BIND_LOCK
        .lock()
        .map_err(|_| "bind Pdfium library lock poisoned".to_string())?;
    let bindings = match std::env::var(PDFIUM_LIBRARY_PATH_ENV) {
        Ok(path) if !path.trim().is_empty() => Pdfium::bind_to_library(path.as_str()),
        _ => Pdfium::bind_to_system_library(),
    };
    match bindings {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(Pdfium::default()),
        Err(error) => Err(format!("bind Pdfium library: {error}")),
    }
}

#[cfg(feature = "pdf-render")]
pub(super) fn render_document_manifests(
    document: &PdfDocument<'_>,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    selected_page_indices: Option<&[i32]>,
) -> Result<Vec<PdfPageShardManifest>, ReportParts> {
    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let shard_dir = context.shard_dir(source_hash);
    fs::create_dir_all(shard_dir.as_path()).map_err(|error| {
        ReportParts::fallback(
            page_count,
            0,
            format!("create shard dir `{}`: {error}", shard_dir.display()),
        )
    })?;

    let page_indices = selected_page_indices.map_or_else(
        || document.pages().as_range().collect::<Vec<_>>(),
        <[i32]>::to_vec,
    );
    page_indices
        .into_iter()
        .enumerate()
        .map(|(rendered_count, page_index)| {
            let rendered_count = checked_len_u32(rendered_count);
            let page = document.pages().get(page_index).map_err(|error| {
                ReportParts::fallback(
                    page_count,
                    rendered_count,
                    format!("load page {page_index}: {error}"),
                )
            })?;
            render_page_manifest(&page, page_index, context, source_hash)
                .map_err(|error| ReportParts::fallback(page_count, rendered_count, error))
        })
        .collect()
}

pub(super) fn source_page_range_document_manifests(
    context: &RenderShardContext<'_>,
    source_hash: &str,
    page_count: u32,
    selected_page_indices: Option<&[i32]>,
) -> Result<Vec<PdfPageShardManifest>, ReportParts> {
    let page_indices = selected_page_indices.map_or_else(
        || source_page_range_all_page_indices(page_count),
        <[i32]>::to_vec,
    );
    page_indices
        .into_iter()
        .enumerate()
        .map(|(rendered_count, page_index)| {
            let rendered_count = checked_len_u32(rendered_count);
            source_page_range_validate_page_index(page_index, page_count)
                .map(|page_index| source_page_range_manifest(page_index, context, source_hash))
                .map_err(|error| ReportParts::fallback(page_count, rendered_count, error))
        })
        .collect()
}

#[cfg(feature = "pdf-render")]
pub(super) fn render_document_region_manifests(
    document: &PdfDocument<'_>,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Vec<PdfPageShardManifest>, ReportParts> {
    let page_count = u32::try_from(document.pages().len()).unwrap_or_default();
    let shard_dir = context.shard_dir(source_hash);
    fs::create_dir_all(shard_dir.as_path()).map_err(|error| {
        ReportParts::fallback(
            page_count,
            0,
            format!("create shard dir `{}`: {error}", shard_dir.display()),
        )
    })?;

    let mut sorted_regions = regions.to_vec();
    sorted_regions.sort_by(|left, right| {
        left.page_index
            .cmp(&right.page_index)
            .then_with(|| {
                left.effective_reading_order_key()
                    .cmp(&right.effective_reading_order_key())
            })
            .then_with(|| left.region_index.cmp(&right.region_index))
    });

    let mut manifests = Vec::new();
    let mut cursor = 0;
    while cursor < sorted_regions.len() {
        let page_index = sorted_regions[cursor].page_index;
        let next_cursor = sorted_regions[cursor..]
            .iter()
            .position(|region| region.page_index != page_index)
            .map_or(sorted_regions.len(), |offset| cursor + offset);
        let page = document
            .pages()
            .get(i32::try_from(page_index).unwrap_or(i32::MAX))
            .map_err(|error| {
                ReportParts::fallback(
                    page_count,
                    checked_len_u32(manifests.len()),
                    format!("load page {page_index}: {error}"),
                )
            })?;
        let page_manifests = render_page_region_manifests(
            &page,
            page_index,
            context,
            source_hash,
            &sorted_regions[cursor..next_cursor],
        )
        .map_err(|error| {
            ReportParts::fallback(page_count, checked_len_u32(manifests.len()), error)
        })?;
        manifests.extend(page_manifests);
        cursor = next_cursor;
    }
    Ok(manifests)
}

#[cfg(feature = "pdf-render")]
fn render_page_manifest(
    page: &PdfPage<'_>,
    page_index: i32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
) -> Result<PdfPageShardManifest, String> {
    let rendered = render_page_image(page, page_index, context.profile)?;
    let image_path = context.shard_dir(source_hash).join(format!(
        "page-{page_index:05}.{}",
        context.profile.image_extension
    ));
    let raster = save_image_identity(&rendered.image, image_path.as_path())?;
    Ok(build_shard_manifest(PdfPageShardManifestInput {
        source_path: context.path,
        source_content_hash: source_hash,
        page_index: u32::try_from(page_index).unwrap_or_default(),
        profile: context.profile,
        media_box: rendered.media_box,
        crop_box: rendered.crop_box,
        rotation_degrees: rendered.rotation_degrees,
        raster,
    }))
}

fn source_page_range_manifest(
    page_index: u32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
) -> PdfPageShardManifest {
    let page_box = PdfPageBox::new(0.0, 0.0, 612.0, 792.0);
    let (raster_width_px, raster_height_px) =
        render_dimensions_for_box(page_box, 0, context.profile);
    let placeholder_path = context.shard_dir(source_hash).join(format!(
        "source-page-range-{page_index:05}.{}",
        context.profile.image_extension
    ));
    let raster = RenderedRasterIdentity {
        path: placeholder_path,
        sha256: sha256_hex(format!("source-page-range:{source_hash}:{page_index}").as_bytes()),
        width_px: raster_width_px,
        height_px: raster_height_px,
    };
    build_shard_manifest(PdfPageShardManifestInput {
        source_path: context.path,
        source_content_hash: source_hash,
        page_index,
        profile: context.profile,
        media_box: page_box,
        crop_box: page_box,
        rotation_degrees: 0,
        raster,
    })
}

#[cfg(feature = "pdf-render")]
fn render_page_region_manifests(
    page: &PdfPage<'_>,
    page_index: u32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Vec<PdfPageShardManifest>, String> {
    let render_mode = pdf_region_render_mode();
    if render_mode == PdfRegionRenderMode::DirectCrop {
        return render_page_region_direct_crop_manifests(
            page,
            page_index,
            context,
            source_hash,
            regions,
        );
    }
    let rendered = render_page_image(
        page,
        i32::try_from(page_index).unwrap_or(i32::MAX),
        context.profile,
    )?;
    let parent_shard_element_id =
        shard_element_id(source_hash, page_index, context.profile.profile_id.as_str());
    regions
        .iter()
        .map(|request| {
            let source_page_pixel_box = region_pixel_box_for_crop(
                rendered.crop_box,
                request.region_box,
                rendered.image.width(),
                rendered.image.height(),
            )?;
            let image_path = context.shard_dir(source_hash).join(format!(
                "page-{page_index:05}-region-{:05}.{}",
                request.region_index, context.profile.image_extension
            ));
            let raster = save_region_crop_image(
                &rendered.image,
                source_page_pixel_box,
                image_path.as_path(),
            )?;
            build_region_shard_manifest(PdfPageRegionShardManifestInput {
                source_path: context.path,
                source_content_hash: source_hash,
                page_index,
                profile: context.profile,
                media_box: rendered.media_box,
                page_crop_box: rendered.crop_box,
                region: PdfPageRegion::new(
                    request.region_index,
                    request.region_box,
                    parent_shard_element_id.clone(),
                    request.effective_reading_order_key(),
                ),
                rotation_degrees: rendered.rotation_degrees,
                page_raster_width_px: rendered.image.width(),
                page_raster_height_px: rendered.image.height(),
                raster,
            })
        })
        .collect()
}

#[cfg(feature = "pdf-render")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdfRegionRenderMode {
    Default,
    DirectCrop,
}

#[cfg(feature = "pdf-render")]
fn pdf_region_render_mode() -> PdfRegionRenderMode {
    pdf_region_render_mode_from_value(std::env::var(PDF_REGION_RENDER_MODE_ENV).ok().as_deref())
}

#[cfg(feature = "pdf-render")]
fn pdf_region_render_mode_from_value(mode: Option<&str>) -> PdfRegionRenderMode {
    let Some(mode) = mode else {
        return PdfRegionRenderMode::Default;
    };
    match mode.trim().replace('_', "-").to_ascii_lowercase().as_str() {
        PDF_REGION_RENDER_MODE_DIRECT_CROP => PdfRegionRenderMode::DirectCrop,
        _ => PdfRegionRenderMode::Default,
    }
}

#[cfg(feature = "pdf-render")]
fn render_page_region_direct_crop_manifests(
    page: &PdfPage<'_>,
    page_index: u32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Vec<PdfPageShardManifest>, String> {
    let page_index_i32 = i32::try_from(page_index).unwrap_or(i32::MAX);
    let geometry = page_geometry(page, page_index_i32, context.profile)?;
    let parent_shard_element_id =
        shard_element_id(source_hash, page_index, context.profile.profile_id.as_str());
    regions
        .iter()
        .map(|request| {
            build_direct_crop_region_manifest(
                page,
                page_index_i32,
                context,
                source_hash,
                request,
                &geometry,
                parent_shard_element_id.as_str(),
            )
        })
        .collect()
}

#[cfg(feature = "pdf-render")]
fn build_direct_crop_region_manifest(
    page: &PdfPage<'_>,
    page_index_i32: i32,
    context: &RenderShardContext<'_>,
    source_hash: &str,
    request: &PdfPageRegionRenderRequest,
    geometry: &PageGeometry,
    parent_shard_element_id: &str,
) -> Result<PdfPageShardManifest, String> {
    let region_box = request
        .region_box
        .intersection(geometry.crop_box)
        .ok_or_else(|| "region does not intersect page crop box".to_string())?;
    let source_page_pixel_box = region_pixel_box_for_crop(
        geometry.crop_box,
        region_box,
        geometry.raster_width_px,
        geometry.raster_height_px,
    )?;
    let image_path = context.shard_dir(source_hash).join(format!(
        "page-{:05}-region-{:05}.{}",
        request.page_index, request.region_index, context.profile.image_extension
    ));
    let raster = render_direct_region_crop_image(
        page,
        page_index_i32,
        context.profile,
        geometry,
        source_page_pixel_box,
        image_path.as_path(),
    )?;
    build_region_shard_manifest(PdfPageRegionShardManifestInput {
        source_path: context.path,
        source_content_hash: source_hash,
        page_index: request.page_index,
        profile: context.profile,
        media_box: geometry.media_box,
        page_crop_box: geometry.crop_box,
        region: PdfPageRegion::new(
            request.region_index,
            region_box,
            parent_shard_element_id,
            request.effective_reading_order_key(),
        ),
        rotation_degrees: geometry.rotation_degrees,
        page_raster_width_px: geometry.raster_width_px,
        page_raster_height_px: geometry.raster_height_px,
        raster,
    })
}

#[cfg(feature = "pdf-render")]
struct RenderedPageImage {
    image: DynamicImage,
    media_box: PdfPageBox,
    crop_box: PdfPageBox,
    rotation_degrees: u16,
}

#[cfg(feature = "pdf-render")]
struct PageGeometry {
    media_box: PdfPageBox,
    crop_box: PdfPageBox,
    rotation_degrees: u16,
    raster_width_px: u32,
    raster_height_px: u32,
}

#[cfg(feature = "pdf-render")]
fn render_page_image(
    page: &PdfPage<'_>,
    page_index: i32,
    profile: &PdfPageRenderProfile,
) -> Result<RenderedPageImage, String> {
    let geometry = page_geometry(page, page_index, profile)?;
    let config = PdfRenderConfig::new()
        .set_target_size(
            checked_pixels_i32(geometry.raster_width_px)?,
            checked_pixels_i32(geometry.raster_height_px)?,
        )
        .set_format(PdfBitmapFormat::BGRA)
        .render_annotations(profile.render_annotations)
        .render_form_data(profile.render_form_data);
    let bitmap = page
        .render_with_config(&config)
        .map_err(|error| format!("render page {page_index}: {error}"))?;
    let image = bitmap
        .as_image()
        .map_err(|error| format!("convert page {page_index} bitmap to image: {error}"))?;
    Ok(RenderedPageImage {
        image,
        media_box: geometry.media_box,
        crop_box: geometry.crop_box,
        rotation_degrees: geometry.rotation_degrees,
    })
}

#[cfg(feature = "pdf-render")]
fn render_direct_region_crop_image(
    page: &PdfPage<'_>,
    page_index: i32,
    profile: &PdfPageRenderProfile,
    geometry: &PageGeometry,
    pixel_box: PdfPagePixelBox,
    image_path: &Path,
) -> Result<RenderedRasterIdentity, String> {
    let scale_x = pdf_pixel_to_f32(geometry.raster_width_px)? / page.width().value.max(1.0);
    let scale_y = pdf_pixel_to_f32(geometry.raster_height_px)? / page.height().value.max(1.0);
    let crop_left = pdf_pixel_to_f32(pixel_box.left)?;
    let crop_top = pdf_pixel_to_f32(pixel_box.top)?;
    let config = PdfRenderConfig::new()
        .set_fixed_size(
            checked_pixels_i32(pixel_box.width_px())?,
            checked_pixels_i32(pixel_box.height_px())?,
        )
        .set_format(PdfBitmapFormat::BGRA)
        .render_annotations(profile.render_annotations)
        .render_form_data(false)
        .transform(scale_x, 0.0, 0.0, scale_y, -crop_left, -crop_top)
        .map_err(|error| format!("configure direct region crop render: {error}"))?;
    let bitmap = page
        .render_with_config(&config)
        .map_err(|error| format!("render direct region crop page {page_index}: {error}"))?;
    let image = bitmap.as_image().map_err(|error| {
        format!("convert direct region crop page {page_index} bitmap to image: {error}")
    })?;
    save_image_identity(&image, image_path)
}

#[cfg(feature = "pdf-render")]
fn pdf_pixel_to_f32(value: u32) -> Result<f32, String> {
    value
        .to_f32()
        .ok_or_else(|| format!("convert PDF pixel value {value} to f32"))
}

#[cfg(feature = "pdf-render")]
fn page_geometry(
    page: &PdfPage<'_>,
    page_index: i32,
    profile: &PdfPageRenderProfile,
) -> Result<PageGeometry, String> {
    let media_box = page.boundaries().media().map_or_else(
        |_| PdfPageBox::from_pdfium_rect(page.page_size()),
        |boundary| PdfPageBox::from_pdfium_rect(boundary.bounds),
    );
    let crop_box = page.boundaries().crop().map_or(media_box, |boundary| {
        PdfPageBox::from_pdfium_rect(boundary.bounds)
    });
    let rotation_degrees = pdf_rotation_degrees_from_result(page.rotation(), page_index)?;
    let (target_width, target_height) =
        render_dimensions_for_box(crop_box, rotation_degrees, profile);
    Ok(PageGeometry {
        media_box,
        crop_box,
        rotation_degrees,
        raster_width_px: target_width,
        raster_height_px: target_height,
    })
}

#[cfg(feature = "pdf-render")]
fn pdf_rotation_degrees_from_result(
    rotation: Result<pdfium_render::prelude::PdfPageRenderRotation, PdfiumError>,
    page_index: i32,
) -> Result<u16, String> {
    match rotation {
        Ok(rotation) => Ok(rotation_to_degrees(rotation)),
        Err(PdfiumError::UnknownBitmapRotation) => Ok(0),
        Err(error) => Err(format!("read page {page_index} rotation: {error}")),
    }
}

#[cfg(feature = "pdf-render")]
fn save_image_identity(
    image: &DynamicImage,
    image_path: &Path,
) -> Result<RenderedRasterIdentity, String> {
    image
        .save(image_path)
        .map_err(|error| format!("write shard image `{}`: {error}", image_path.display()))?;
    let raster_bytes = fs::read(image_path)
        .map_err(|error| format!("read shard image `{}`: {error}", image_path.display()))?;
    Ok(RenderedRasterIdentity {
        path: image_path.to_path_buf(),
        sha256: sha256_hex(&raster_bytes),
        width_px: image.width(),
        height_px: image.height(),
    })
}

#[cfg(feature = "pdf-render")]
pub(super) fn save_region_crop_image(
    page_image: &DynamicImage,
    pixel_box: PdfPagePixelBox,
    image_path: &Path,
) -> Result<RenderedRasterIdentity, String> {
    if pixel_box.right > page_image.width() || pixel_box.bottom > page_image.height() {
        return Err(format!(
            "region pixel box exceeds source raster: box=({}, {}, {}, {}), raster={}x{}",
            pixel_box.left,
            pixel_box.top,
            pixel_box.right,
            pixel_box.bottom,
            page_image.width(),
            page_image.height()
        ));
    }
    let crop = page_image.crop_imm(
        pixel_box.left,
        pixel_box.top,
        pixel_box.width_px(),
        pixel_box.height_px(),
    );
    save_image_identity(&crop, image_path)
}

#[cfg(all(test, feature = "pdf-render"))]
#[path = "../../../tests/unit/pdf/render/document.rs"]
mod tests;
