use super::{
    HOSTED_VLM_REGION_DISPATCH_CHUNK_SIZE_ENV, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    endpoint_index_for_request, rendered_region_dispatch_chunk_size_with_lookup,
    rendered_region_shard_chunks, rendered_region_shard_chunks_with_composite_size,
    sample_ocr_input, source_pdf_page_range_chunk_endpoint_index_with_lookup,
    source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup, source_pdf_page_range_chunks,
    source_pdf_page_range_chunks_with_fast_text_split, source_pdf_page_range_chunks_with_weights,
    source_pdf_page_range_dispatch_budget,
    source_pdf_page_range_dispatch_budget_with_region_pipeline,
    source_pdf_page_range_dispatch_budget_with_region_pipeline_and_fast_text_split,
    source_pdf_page_range_dispatch_chunks,
};

include!("ranges.rs");
include!("dispatch.rs");
include!("regions.rs");
include!("endpoints.rs");
