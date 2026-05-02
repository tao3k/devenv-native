"""Low-level Docling element value accessors."""

from __future__ import annotations

from typing import Any


def _element_content(element: Any) -> str:
    for method_name in ("export_to_markdown", "export_to_html", "export_to_text"):
        method = getattr(element, method_name, None)
        if callable(method):
            try:
                value = method()
            except TypeError:
                continue
            if value:
                return str(value)
    for attribute_name in ("text", "content", "transcript", "caption", "label"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value)
    if isinstance(element, str):
        return element
    return ""


def _element_resource_path(element: Any) -> str:
    for attribute_name in ("resource_path", "path", "uri"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value)
    image = getattr(element, "image", None)
    if image is not None:
        uri = getattr(image, "uri", None)
        if uri:
            return str(uri)
    return ""


def _element_caption(element: Any) -> str:
    value = getattr(element, "caption", "")
    if isinstance(value, list):
        return " ".join(str(item) for item in value if item)
    return str(value) if value else ""


def _element_page_index(element: Any) -> int:
    page_no = getattr(element, "page_no", None)
    if page_no is None:
        provenance = getattr(element, "prov", None) or getattr(
            element, "provenance", None
        )
        if provenance:
            first = (
                provenance[0] if isinstance(provenance, (list, tuple)) else provenance
            )
            page_no = getattr(first, "page_no", None)
    try:
        return max(int(page_no) - 1, 0) if page_no is not None else 0
    except (TypeError, ValueError):
        return 0


def _element_confidence(element: Any) -> float | None:
    value = getattr(element, "confidence", None)
    if value is None:
        value = getattr(element, "score", None)
    try:
        return float(value) if value is not None else None
    except (TypeError, ValueError):
        return None


def _element_bbox(element: Any) -> tuple[float, float, float, float] | None:
    bbox = getattr(element, "bbox", None)
    if bbox is None:
        provenance = getattr(element, "prov", None) or getattr(
            element, "provenance", None
        )
        if provenance:
            first = (
                provenance[0] if isinstance(provenance, (list, tuple)) else provenance
            )
            bbox = getattr(first, "bbox", None)
    if bbox is None:
        return None
    values = (
        _bbox_value(bbox, ("l", "left", "x0")),
        _bbox_value(bbox, ("t", "top", "y0")),
        _bbox_value(bbox, ("r", "right", "x1")),
        _bbox_value(bbox, ("b", "bottom", "y1")),
    )
    if any(value is None for value in values):
        return None
    left, top, right, bottom = values
    assert left is not None
    assert top is not None
    assert right is not None
    assert bottom is not None
    return (left, top, right, bottom)


def _bbox_value(bbox: Any, names: tuple[str, ...]) -> float | None:
    for name in names:
        value = getattr(bbox, name, None)
        if value is not None:
            try:
                return float(value)
            except (TypeError, ValueError):
                return None
    return None


def _element_provenance(element: Any) -> object:
    provenance = getattr(element, "prov", None) or getattr(element, "provenance", None)
    if provenance is None:
        return {"source": "docling_element"}
    if isinstance(provenance, (str, int, float, bool)):
        return {"value": provenance}
    if isinstance(provenance, (list, tuple)):
        return [_safe_mapping(item) for item in provenance]
    return _safe_mapping(provenance)


def _safe_mapping(value: Any) -> object:
    if isinstance(value, dict):
        return {str(key): _safe_scalar(item) for key, item in value.items()}
    mapping: dict[str, object] = {}
    for key in ("page_no", "self_ref", "cref", "id"):
        item = getattr(value, key, None)
        if item is not None:
            mapping[key] = _safe_scalar(item)
    bbox = getattr(value, "bbox", None)
    if bbox is not None:
        mapping["bbox"] = {
            name: _bbox_value(bbox, (name,))
            for name in ("l", "t", "r", "b")
            if _bbox_value(bbox, (name,)) is not None
        }
    return mapping or str(value)


def _safe_scalar(value: Any) -> object:
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    return str(value)
