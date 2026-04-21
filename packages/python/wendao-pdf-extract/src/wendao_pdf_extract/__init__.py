"""Wendao PDF extraction service using OpenDataLoader via Arrow Flight."""

from ._version import __version__
from .extractor import extract_pdf
from .server import PdfExtractFlightServer, main

__all__ = [
    "__version__",
    "extract_pdf",
    "main",
    "PdfExtractFlightServer",
]
