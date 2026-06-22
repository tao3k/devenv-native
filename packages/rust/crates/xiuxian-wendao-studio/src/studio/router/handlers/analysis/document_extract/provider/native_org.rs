use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc as StdArc;

use arrow::array::{Array, ArrayRef, StringArray};
use arrow::record_batch::RecordBatch;
use futures::stream::{self, StreamExt};
use xiuxian_wendao_parsers::{
    OrgAttachmentLink, OrgAttachmentLinkProtocol, extract_org_attachment_links,
};
use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FULL_PROFILE, DocumentExtractFlightRouteResponse,
};

use super::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, build_native_text_resource_batch, write_arrow_file,
};

impl StudioDocumentExtractFlightRouteProvider {
    pub(super) async fn sync_native_org_document_extract_batch(
        &self,
        source: &Path,
        output: &Path,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let batches = self
            .write_native_org_document_extract_output(source, output, force, error_row)
            .await?;
        write_arrow_file(
            output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path(),
            batches.as_slice(),
        )?;
        tokio::fs::File::create(output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch native Org document extract marker: {error}"))?;
        if let Err(error) = self.persist_sync_output_artifact(source, output).await {
            log::warn!("failed to persist native Org document extract artifact: {error}");
        }
        Ok(DocumentExtractFlightRouteResponse::from_batches(batches))
    }

    pub(super) async fn write_native_org_document_extract_output(
        &self,
        source: &Path,
        output: &Path,
        force: bool,
        error_row: bool,
    ) -> Result<Vec<RecordBatch>, String> {
        tokio::fs::create_dir_all(output).await.map_err(|error| {
            format!(
                "create native Org document extract output `{}`: {error}",
                output.display()
            )
        })?;
        let content = tokio::fs::read_to_string(source)
            .await
            .map_err(|error| format!("read native Org document `{}`: {error}", source.display()))?;
        if content.trim().is_empty() {
            return Err(format!(
                "native Org document `{}` produced no text content",
                source.display()
            ));
        }
        let file_name = source.file_name().ok_or_else(|| {
            format!(
                "native Org document `{}` does not have a file name",
                source.display()
            )
        })?;
        let resource_path = output.join(file_name);
        tokio::fs::write(resource_path.as_path(), content.as_bytes())
            .await
            .map_err(|error| {
                format!(
                    "write native Org document resource `{}`: {error}",
                    resource_path.display()
                )
            })?;
        let source_path = source.to_string_lossy();
        let resource_path = resource_path.to_string_lossy();
        let mut batches = vec![build_native_text_resource_batch(
            source_path.as_ref(),
            "org-document",
            resource_path.as_ref(),
            "Org document",
            content.as_str(),
            "text/org",
            "_org_document",
        )?];

        let (attachment_groups, analysis_requests) =
            collect_native_org_attachment_groups(source, output, content.as_str())?;

        let mut analysis_batches = self
            .analyze_native_org_attachments(analysis_requests, force, error_row)
            .await?;
        batches.extend(native_org_attachment_batches(
            attachment_groups,
            &mut analysis_batches,
        ));

        Ok(batches)
    }

    async fn analyze_native_org_attachments(
        &self,
        requests: Vec<NativeOrgAttachmentAnalysisRequest>,
        force: bool,
        error_row: bool,
    ) -> Result<BTreeMap<usize, Vec<RecordBatch>>, String> {
        let mut pending = stream::iter(requests.into_iter().map(|request| async move {
            let batches = self
                .request_python_document_extract(
                    request.source_path.as_str(),
                    request.output_dir.as_str(),
                    force,
                    error_row,
                    DOCUMENT_EXTRACT_FULL_PROFILE,
                )
                .await?;
            Ok::<_, String>((
                request.index,
                namespace_native_org_attachment_batches(request.index, batches)?,
            ))
        }))
        .buffer_unordered(self.runtime.conversion_limit.max(1));

        let mut completed = BTreeMap::new();
        while let Some(result) = pending.next().await {
            let (index, batches) = result?;
            completed.insert(index, batches);
        }
        Ok(completed)
    }
}

struct NativeOrgAttachmentGroup {
    link_batch: RecordBatch,
    analysis: Option<NativeOrgAttachmentAnalysisRequest>,
}

#[derive(Clone)]
struct NativeOrgAttachmentAnalysisRequest {
    index: usize,
    source_path: String,
    output_dir: String,
}

fn collect_native_org_attachment_groups(
    source: &Path,
    output: &Path,
    content: &str,
) -> Result<
    (
        Vec<NativeOrgAttachmentGroup>,
        Vec<NativeOrgAttachmentAnalysisRequest>,
    ),
    String,
> {
    let attachment_groups = extract_org_attachment_links(content)
        .iter()
        .enumerate()
        .map(|(index, link)| {
            let resolved = resolve_org_attachment_source(source, link);
            let link_batch =
                build_org_attachment_link_resource_batch(source, link, index, resolved.as_deref())?;
            let analysis = resolved
                .filter(|path| {
                    path.is_file()
                        && !is_native_org_source(path.as_path())
                        && should_analyze_org_attachment(path.as_path())
                })
                .map(|attachment_source| NativeOrgAttachmentAnalysisRequest {
                    index,
                    source_path: attachment_source.to_string_lossy().to_string(),
                    output_dir: output
                        .join("_org_attachments")
                        .join(format!("attachment-{index:04}"))
                        .to_string_lossy()
                        .to_string(),
                });

            Ok(NativeOrgAttachmentGroup {
                link_batch,
                analysis,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let analysis_requests = attachment_groups
        .iter()
        .filter_map(|group| group.analysis.clone())
        .collect();
    Ok((attachment_groups, analysis_requests))
}

fn native_org_attachment_batches(
    attachment_groups: Vec<NativeOrgAttachmentGroup>,
    analysis_batches: &mut BTreeMap<usize, Vec<RecordBatch>>,
) -> Vec<RecordBatch> {
    attachment_groups
        .into_iter()
        .flat_map(|group| {
            let mut batches = vec![group.link_batch];
            if let Some(request) = group.analysis
                && let Some(mut attachment_batches) = analysis_batches.remove(&request.index)
            {
                batches.append(&mut attachment_batches);
            }
            batches
        })
        .collect()
}

fn namespace_native_org_attachment_batches(
    index: usize,
    batches: Vec<RecordBatch>,
) -> Result<Vec<RecordBatch>, String> {
    batches
        .into_iter()
        .map(|batch| namespace_native_org_attachment_batch(index, &batch))
        .collect()
}

fn namespace_native_org_attachment_batch(
    index: usize,
    batch: &RecordBatch,
) -> Result<RecordBatch, String> {
    let element_id_index = batch
        .schema()
        .index_of("elementId")
        .map_err(|error| format!("analyzer attachment batch missing elementId: {error}"))?;
    let element_ids = batch
        .column(element_id_index)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| "analyzer attachment elementId column is not Utf8".to_string())?;
    let namespaced = (0..batch.num_rows())
        .map(|row| {
            (!element_ids.is_null(row)).then(|| {
                format!(
                    "_org_attachment_{index:04}_{}",
                    element_ids.value(row).trim_start_matches('_')
                )
            })
        })
        .collect::<Vec<_>>();
    let mut columns = batch.columns().to_vec();
    columns[element_id_index] = StdArc::new(StringArray::from(namespaced)) as ArrayRef;
    RecordBatch::try_new(batch.schema(), columns).map_err(|error| error.to_string())
}

pub(super) fn is_native_org_source(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

fn resolve_org_attachment_source(source: &Path, link: &OrgAttachmentLink) -> Option<PathBuf> {
    let target = Path::new(link.target_path.as_str());
    if target.is_absolute() {
        return Some(target.to_path_buf());
    }
    let base = source.parent()?;
    match link.protocol {
        OrgAttachmentLinkProtocol::File => Some(base.join(target)),
        OrgAttachmentLinkProtocol::Attachment => {
            let sibling = base.join(target);
            if sibling.exists() {
                return Some(sibling);
            }
            let data_child = base.join("data").join(target);
            data_child.exists().then_some(data_child)
        }
    }
}

fn build_org_attachment_link_resource_batch(
    source: &Path,
    link: &OrgAttachmentLink,
    index: usize,
    resolved: Option<&Path>,
) -> Result<RecordBatch, String> {
    let source_path = source.to_string_lossy();
    let resource_path = resolved.map_or_else(
        || link.target_path.clone(),
        |path| path.to_string_lossy().to_string(),
    );
    let caption = org_attachment_caption(link, index);
    let metadata = serde_json::json!({
        "schema": "xiuxian_wendao.org_attachment_link.v1",
        "protocol": match link.protocol {
            OrgAttachmentLinkProtocol::File => "file",
            OrgAttachmentLinkProtocol::Attachment => "attachment",
        },
        "rawPath": link.raw_path.as_str(),
        "targetPath": link.target_path.as_str(),
        "line": link.line,
        "resolved": resolved.is_some_and(Path::exists),
        "analyzerEligible": resolved.is_some_and(should_analyze_org_attachment),
    })
    .to_string();

    build_native_text_resource_batch(
        source_path.as_ref(),
        "org-attachment-link",
        resource_path.as_str(),
        caption.as_str(),
        metadata.as_str(),
        "application/vnd.xiuxian.org-attachment-link+json",
        format!("_org_attachment_link_{index:04}").as_str(),
    )
}

fn org_attachment_caption(link: &OrgAttachmentLink, index: usize) -> String {
    let description = link.description.trim();
    if !description.is_empty() {
        return description.to_string();
    }
    link.caption
        .as_deref()
        .map(str::trim)
        .filter(|caption| !caption.is_empty())
        .map_or_else(
            || format!("Org attachment link {}", index + 1),
            ToString::to_string,
        )
}

fn should_analyze_org_attachment(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "pdf"
            | "docx"
            | "pptx"
            | "html"
            | "htm"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "tiff"
            | "tif"
            | "webp"
            | "avif"
            | "svg"
            | "md"
            | "csv"
            | "xlsx"
            | "xml"
            | "mp3"
            | "wav"
            | "m4a"
            | "flac"
            | "vtt"
            | "tex"
            | "latex"
            | "adoc"
            | "asciidoc"
    )
}
