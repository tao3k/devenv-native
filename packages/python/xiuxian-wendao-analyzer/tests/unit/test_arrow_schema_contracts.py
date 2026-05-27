import pyarrow as pa

from xiuxian_wendao_analyzer.arrow_schema_contracts import (
    ArrowSchemaColumn,
    build_arrow_schema,
    schema_table_name,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reference_pack import (
    REFERENCE_SELECTION_REVIEW_TABLE,
)
from xiuxian_wendao_analyzer.audio_shard_contracts import (
    AUDIO_SHARD_INPUT_SCHEMA,
    AUDIO_SHARD_RESULT_SCHEMA,
)
from xiuxian_wendao_analyzer.document_metrics import DOCUMENT_TIMING_SCHEMA
from xiuxian_wendao_analyzer.document_types import (
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA,
)
from xiuxian_wendao_analyzer.pdf_ocr_contracts import (
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA,
)


def test_build_arrow_schema_attaches_wendao_table_metadata() -> None:
    schema = build_arrow_schema(
        "analyzer_test_table",
        (
            ArrowSchemaColumn("id", pa.utf8(), nullable=False),
            ArrowSchemaColumn("payload", pa.binary()),
        ),
    )

    assert schema_table_name(schema) == "analyzer_test_table"
    assert schema.field("id").nullable is False
    assert schema.field("payload").type == pa.binary()


def test_document_schemas_use_analyzer_contract_metadata() -> None:
    assert schema_table_name(DOCUMENT_RESOURCE_SCHEMA) == "document_resource"
    assert schema_table_name(DOCUMENT_STRUCTURE_SCHEMA) == "pdf_document_structure"


def test_worker_and_timing_schemas_use_analyzer_contract_metadata() -> None:
    assert schema_table_name(PDF_OCR_SHARD_INPUT_SCHEMA) == "pdf_ocr_shard_input"
    assert schema_table_name(PDF_OCR_SHARD_RESULT_SCHEMA) == "pdf_ocr_shard_result"
    assert schema_table_name(AUDIO_SHARD_INPUT_SCHEMA) == "audio_shard_input"
    assert schema_table_name(AUDIO_SHARD_RESULT_SCHEMA) == "audio_shard_result"
    assert schema_table_name(DOCUMENT_TIMING_SCHEMA) == "document_timing"


def test_audio_reference_review_schema_uses_analyzer_contract_metadata() -> None:
    assert schema_table_name(REFERENCE_SELECTION_REVIEW_TABLE) == "audio_reference_selection_review"
