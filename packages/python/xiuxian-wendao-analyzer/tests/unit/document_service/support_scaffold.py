"""Hosted VLM scaffold fixtures for document service tests."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def write_ocr2_region_scaffold_sidecar(
    directory: Path,
    rows: list[dict[str, object]],
    *,
    raster_sha256: str = "rasterhash",
) -> None:
    """Write a hosted VLM region scaffold sidecar for test rows."""

    (directory / "_hosted_vlm_region_scaffolds.json").write_text(
        json.dumps(
            {
                "schema": "xiuxian_wendao.hosted_vlm_region_scaffold.v1",
                "mode": "region-table-json",
                "items": [_scaffold_item(row, raster_sha256) for row in rows],
            }
        ),
        encoding="utf-8",
    )


def _scaffold_item(
    row: dict[str, object],
    raster_sha256: str,
) -> dict[str, object]:
    return {
        "scaffoldKind": "table_candidate",
        "shardElementId": row["shardElementId"],
        "parentShardElementId": row["parentShardElementId"],
        "pageIndex": row["pageIndex"],
        "regionIndex": row["regionIndex"],
        "sourceContentHash": row["sourceContentHash"],
        "rasterSha256": raster_sha256,
        "renderDpi": row["renderDpi"],
        "cropBox": {
            "left": row["cropLeft"],
            "bottom": row["cropBottom"],
            "right": row["cropRight"],
            "top": row["cropTop"],
        },
        "sourcePagePixelBox": {
            "left": row["sourcePagePixelLeft"],
            "top": row["sourcePagePixelTop"],
            "right": row["sourcePagePixelRight"],
            "bottom": row["sourcePagePixelBottom"],
        },
        "sourcePageProfile": None,
    }


INVALID_REGION_SCAFFOLD_CASES = [
    ("missing sidecar", None, None),
    ("fingerprint mismatch", "wrong-raster", None),
    ("malformed json", "rasterhash", "{not-json"),
    (
        "missing marker",
        "rasterhash",
        json.dumps(
            {
                "regions": [
                    {"marker": "wrong", "shardElementId": "region-a", "text": "x"}
                ]
            }
        ),
    ),
    ("row mismatch", "rasterhash", json.dumps({"regions": []})),
    (
        "empty output",
        "rasterhash",
        json.dumps(
            {
                "regions": [
                    {
                        "marker": (
                            "<!-- xiuxian-wendao-hosted-vlm-region:0:1:region-a -->"
                        ),
                        "shardElementId": "region-a",
                        "text": "",
                        "tables": [],
                    }
                ]
            }
        ),
    ),
    (
        "invalid table shape",
        "rasterhash",
        json.dumps(
            {
                "regions": [
                    {
                        "marker": (
                            "<!-- xiuxian-wendao-hosted-vlm-region:0:1:region-a -->"
                        ),
                        "shardElementId": "region-a",
                        "tables": [{"rows": [["A", "B"], ["1"]]}],
                    }
                ]
            }
        ),
    ),
]
