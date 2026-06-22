use sha2::{Digest, Sha256};
use xiuxian_wendao_episteme::EpistemeCacheTask;

mod docling_document;
mod image_ocr;
mod legacy_office;

fn single_pixel_png_bytes() -> [u8; 24] {
    [
        0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0x00, 0x00, 0x00, 0x0d, b'I', b'H',
        b'D', b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    ]
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn image_task(
    queue_id: &str,
    relative_path: &str,
    source_sha256: String,
    planned_output_path: &str,
) -> EpistemeCacheTask {
    EpistemeCacheTask {
        queue_id: queue_id.to_string(),
        file_id: format!("synthetic.file.{queue_id}"),
        relative_path: relative_path.to_string(),
        category: "synthetic".into(),
        language: "zh-CN".to_string(),
        extraction_route: "image_ocr_evidence".to_string(),
        priority: 10,
        source_sha256,
        planned_output_path: planned_output_path.to_string(),
        output_contract: "cache_only_no_rdf_promotion".to_string(),
        status: "planned".into(),
    }
}

fn docling_task(
    queue_id: &str,
    source_sha256: String,
    planned_output_path: &str,
) -> EpistemeCacheTask {
    EpistemeCacheTask {
        queue_id: queue_id.to_string(),
        file_id: format!("synthetic.file.{queue_id}"),
        relative_path: "docs/evidence.docx".to_string(),
        category: "synthetic".into(),
        language: "zh-CN".to_string(),
        extraction_route: "document_text_evidence".to_string(),
        priority: 10,
        source_sha256,
        planned_output_path: planned_output_path.to_string(),
        output_contract: "cache_only_no_rdf_promotion".to_string(),
        status: "planned".into(),
    }
}

fn legacy_office_task(
    queue_id: &str,
    relative_path: &str,
    source_sha256: String,
    route: &str,
    planned_output_path: &str,
) -> EpistemeCacheTask {
    EpistemeCacheTask {
        queue_id: queue_id.to_string(),
        file_id: format!("synthetic.file.{queue_id}"),
        relative_path: relative_path.to_string(),
        category: "synthetic".into(),
        language: "zh-CN".to_string(),
        extraction_route: route.to_string(),
        priority: 10,
        source_sha256,
        planned_output_path: planned_output_path.to_string(),
        output_contract: "cache_only_no_rdf_promotion".to_string(),
        status: "planned".into(),
    }
}
