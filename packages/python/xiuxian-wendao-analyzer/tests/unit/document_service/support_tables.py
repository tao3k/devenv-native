"""Sample Arrow table builders for document service tests."""

from __future__ import annotations

import pyarrow as pa

from xiuxian_wendao_analyzer import (
    AUDIO_SHARD_INPUT_SCHEMA,
    AUDIO_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
)


def _sample_pdf_ocr_input_table(
    image_path: str = "/tmp/page-00000.png",
    *,
    source_path: str = "/tmp/source.pdf",
    page_index: int = 0,
    shard_element_id: str = "shard-id",
    shard_type: str = "page",
    region_index: int = 0,
    parent_shard_element_id: str = "",
    reading_order_key: str = "000000.000000",
    ocr_profile: str = "docling-compatible-page-ocr-v1",
):
    return pa.Table.from_pylist(
        [
            {
                "contractVersion": PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
                "sourcePath": source_path,
                "sourceContentHash": "sourcehash",
                "pageIndex": page_index,
                "imagePath": image_path,
                "imageMimeType": "image/png",
                "rasterSha256": "rasterhash",
                "renderProfile": "pdfium-render-page-shards-v1",
                "ocrProfile": ocr_profile,
                "ocrEngine": "docling-compatible-ocr",
                "preferredLanguages": "auto",
                "minConfidence": 0.0,
                "preserveLayout": True,
                "rasterWidthPx": 2400,
                "rasterHeightPx": 3100,
                "renderDpi": 300,
                "rotationDegrees": 0,
                "cropLeft": 0.0,
                "cropBottom": 0.0,
                "cropRight": 612.0,
                "cropTop": 792.0,
                "pointToPixelScaleX": 3.921568627,
                "pointToPixelScaleY": 3.914141414,
                "shardElementId": shard_element_id,
                "shardType": shard_type,
                "regionIndex": region_index,
                "parentShardElementId": parent_shard_element_id,
                "readingOrderKey": reading_order_key,
                "sourcePagePixelLeft": 0,
                "sourcePagePixelTop": 0,
                "sourcePagePixelRight": 2400,
                "sourcePagePixelBottom": 3100,
            }
        ],
        schema=PDF_OCR_SHARD_INPUT_SCHEMA,
    )


def _sample_audio_shard_input_table(
    shard_path: str = "/tmp/chunk.wav",
    *,
    source_path: str = "/tmp/source.mp3",
    shard_element_id: str = "audio-shard-id",
):
    return pa.Table.from_pylist(
        [
            {
                "contractVersion": AUDIO_SHARD_INPUT_SCHEMA_VERSION,
                "sourcePath": source_path,
                "sourceContentHash": "sourcehash",
                "shardPath": shard_path,
                "shardSha256": "shardhash",
                "shardProfile": "audio-shards-v1",
                "taskProfile": "transcription",
                "backendProfile": "hosted-audio-transcript-v1",
                "preferredLanguages": "zh",
                "sampleRateHz": 16000,
                "channels": 1,
                "audioFormat": "wav",
                "startMs": 0,
                "durationMs": 30000,
                "mediaStartMs": 0,
                "mediaDurationMs": 30000,
                "contextBeforeMs": 0,
                "contextAfterMs": 0,
                "shardElementId": shard_element_id,
                "readingOrderKey": "000000.000000000000",
            }
        ],
        schema=AUDIO_SHARD_INPUT_SCHEMA,
    )
