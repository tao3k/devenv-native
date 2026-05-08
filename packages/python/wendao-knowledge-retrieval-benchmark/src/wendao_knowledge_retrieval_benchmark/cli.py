"""CLI for Wendao knowledge retrieval black-box benchmark reports."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .benchmark import build_benchmark_report
from .reporting import render_markdown


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare Wendao knowledge retrieval profiles from a real-repo receipt."
    )
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--output-json", type=Path)
    parser.add_argument("--output-markdown", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    report = build_benchmark_report(args.receipt)
    payload = report.to_json()
    markdown = render_markdown(report)

    if args.output_json:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    if args.output_markdown:
        args.output_markdown.parent.mkdir(parents=True, exist_ok=True)
        args.output_markdown.write_text(markdown, encoding="utf-8")
    if not args.output_json and not args.output_markdown:
        sys.stdout.write(markdown)
    return 0
