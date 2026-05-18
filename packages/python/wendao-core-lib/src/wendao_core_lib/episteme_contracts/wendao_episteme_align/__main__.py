from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.paths import cache_root, episteme_root


DEFAULT_CHUNK_CHARS = 6000
FIRECRAWL_SCRAPE_URL = "https://api.firecrawl.dev/v2/scrape"
FETCHABLE_SOURCE_TYPES = {"paper", "website"}
LOCAL_SOURCE_TYPES = {"local_doc", "local_file"}


@dataclass(frozen=True)
class SourceEntry:
    id: str
    name: str
    url: str
    source_type: str
    enabled: bool
    entrypoints: tuple[str, ...]
    authority: str
    notes: str
    path: str


@dataclass(frozen=True)
class FetchedDocument:
    source_id: str
    url: str
    title: str
    markdown: str
    raw_path: Path
    normalized_path: Path


def framework_cache_dir(framework: str) -> Path:
    return cache_root() / "episteme" / "alignment" / framework


def load_source_entries(framework: str) -> tuple[Path, list[SourceEntry]]:
    sources_root = episteme_root() / "sources"
    for sources_path in sorted(sources_root.glob("*/sources.toml")):
        data = tomllib.loads(sources_path.read_text())
        if data.get("framework") != framework:
            continue
        entries = [
            SourceEntry(
                id=str(source["id"]),
                name=str(source.get("name", source["id"])),
                url=str(source.get("url") or source.get("path", "")),
                source_type=str(source.get("type", "")),
                enabled=bool(source.get("enabled", True)),
                entrypoints=tuple(str(item) for item in source.get("entrypoints", [])),
                authority=str(source.get("authority", "")),
                notes=str(source.get("notes", "")),
                path=str(source.get("path", "")),
            )
            for source in data.get("source", [])
        ]
        return sources_path, entries
    raise SystemExit(f"no sources.toml entry found for framework: {framework}")


def entrypoint_urls(source: SourceEntry) -> list[str]:
    if not source.entrypoints:
        return [source.url]
    parsed_base = urllib.parse.urlparse(source.url)
    base_origin = f"{parsed_base.scheme}://{parsed_base.netloc}"
    urls = []
    for entrypoint in source.entrypoints:
        parsed_entry = urllib.parse.urlparse(entrypoint)
        if parsed_entry.scheme:
            urls.append(entrypoint)
        elif entrypoint.startswith("/"):
            urls.append(urllib.parse.urljoin(base_origin, entrypoint))
        else:
            urls.append(urllib.parse.urljoin(source.url, entrypoint))
    return urls


def safe_name(value: str) -> str:
    value = re.sub(r"^https?://", "", value)
    value = re.sub(r"[^A-Za-z0-9._-]+", "_", value)
    return value.strip("_") or "source"


def is_fetchable_source_type(source_type: str) -> bool:
    return source_type in FETCHABLE_SOURCE_TYPES


def is_local_source_type(source_type: str) -> bool:
    return source_type in LOCAL_SOURCE_TYPES


def sanitize_error_body(body: str) -> str:
    body = re.sub(r"fc-[A-Za-z0-9_*.-]+", "fc-[redacted]", body)
    return body


def firecrawl_scrape(url: str, api_key: str) -> dict[str, Any]:
    payload = json.dumps(
        {
            "url": url,
            "formats": ["markdown"],
            "onlyMainContent": True,
        }
    ).encode()
    request = urllib.request.Request(
        FIRECRAWL_SCRAPE_URL,
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            return json.loads(response.read().decode())
    except urllib.error.HTTPError as error:
        body = sanitize_error_body(error.read().decode(errors="replace"))
        raise SystemExit(
            f"firecrawl scrape failed for {url}: HTTP {error.code}: {body}"
        ) from error


def extract_markdown(payload: dict[str, Any]) -> tuple[str, str]:
    data = payload.get("data")
    if isinstance(data, dict):
        markdown = data.get("markdown") or ""
        metadata = data.get("metadata") or {}
    else:
        markdown = payload.get("markdown") or ""
        metadata = payload.get("metadata") or {}
    title = ""
    if isinstance(metadata, dict):
        title = str(metadata.get("title") or metadata.get("ogTitle") or "")
    if not title:
        match = re.search(r"^#\s+(.+)$", markdown, flags=re.MULTILINE)
        title = match.group(1).strip() if match else ""
    return markdown, title


def write_normalized_markdown(
    framework: str,
    source: SourceEntry,
    url: str,
    title: str,
    markdown: str,
    normalized_path: Path,
    fetcher: str,
) -> None:
    normalized_path.parent.mkdir(parents=True, exist_ok=True)
    header = [
        "---",
        f"framework: {json.dumps(framework)}",
        f"source_id: {json.dumps(source.id)}",
        f"source_name: {json.dumps(source.name)}",
        f"url: {json.dumps(url)}",
        f"title: {json.dumps(title)}",
        f"authority: {json.dumps(source.authority)}",
        f"fetcher: {json.dumps(fetcher)}",
        "---",
        "",
    ]
    normalized_path.write_text("\n".join(header) + markdown)


def fetch_with_firecrawl(
    framework: str, entries: list[SourceEntry]
) -> list[FetchedDocument]:
    firecrawl_entries = [
        source
        for source in entries
        if source.enabled and is_fetchable_source_type(source.source_type)
    ]
    if not firecrawl_entries:
        return []
    api_key = os.environ.get("FIRECRAWL_API_KEY")
    if not api_key:
        raise SystemExit("FIRECRAWL_API_KEY is not set")
    root = framework_cache_dir(framework)
    fetched = []
    for source in firecrawl_entries:
        for url in entrypoint_urls(source):
            name = safe_name(url)
            raw_path = root / "raw" / "firecrawl" / source.id / f"{name}.json"
            normalized_path = root / "normalized" / source.id / f"{name}.md"
            raw_path.parent.mkdir(parents=True, exist_ok=True)
            payload = firecrawl_scrape(url, api_key)
            raw_path.write_text(
                json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
            )
            markdown, title = extract_markdown(payload)
            write_normalized_markdown(
                framework,
                source,
                url,
                title,
                markdown,
                normalized_path,
                "firecrawl",
            )
            fetched.append(
                FetchedDocument(
                    source_id=source.id,
                    url=url,
                    title=title,
                    markdown=markdown,
                    raw_path=raw_path,
                    normalized_path=normalized_path,
                )
            )
    return fetched


def resolve_local_source_path(source: SourceEntry) -> Path:
    if not source.path:
        raise SystemExit(f"local source `{source.id}` is missing path")
    path = Path(source.path)
    if path.is_absolute():
        return path
    episteme_relative = episteme_root() / path
    if episteme_relative.exists():
        return episteme_relative
    return episteme_root().parent / path


def fetch_local_docs(
    framework: str, entries: list[SourceEntry]
) -> list[FetchedDocument]:
    root = framework_cache_dir(framework)
    fetched = []
    for source in entries:
        if not source.enabled or not is_local_source_type(source.source_type):
            continue
        local_path = resolve_local_source_path(source)
        if not local_path.exists():
            raise SystemExit(f"local source `{source.id}` does not exist: {local_path}")
        markdown = local_path.read_text()
        title = extract_markdown({"markdown": markdown})[1] or source.name
        name = safe_name(source.path)
        raw_path = root / "raw" / "local_doc" / source.id / f"{name}.md"
        normalized_path = root / "normalized" / source.id / f"{name}.md"
        raw_path.parent.mkdir(parents=True, exist_ok=True)
        raw_path.write_text(markdown)
        write_normalized_markdown(
            framework,
            source,
            source.path,
            title,
            markdown,
            normalized_path,
            "local_doc",
        )
        fetched.append(
            FetchedDocument(
                source_id=source.id,
                url=source.path,
                title=title,
                markdown=markdown,
                raw_path=raw_path,
                normalized_path=normalized_path,
            )
        )
    return fetched


def chunk_text(markdown: str, chunk_chars: int = DEFAULT_CHUNK_CHARS) -> list[str]:
    paragraphs = re.split(r"\n(?=# |\n)", markdown)
    chunks: list[str] = []
    current = ""
    for paragraph in paragraphs:
        paragraph = paragraph.strip()
        if not paragraph:
            continue
        if current and len(current) + len(paragraph) + 2 > chunk_chars:
            chunks.append(current.strip())
            current = ""
        if len(paragraph) > chunk_chars:
            for start in range(0, len(paragraph), chunk_chars):
                chunks.append(paragraph[start : start + chunk_chars].strip())
        else:
            current = f"{current}\n\n{paragraph}" if current else paragraph
    if current:
        chunks.append(current.strip())
    return chunks


def write_chunks(framework: str, documents: list[FetchedDocument]) -> Path:
    chunks_path = framework_cache_dir(framework) / "chunks" / "source_chunks.jsonl"
    chunks_path.parent.mkdir(parents=True, exist_ok=True)
    with chunks_path.open("w") as handle:
        for document in documents:
            for index, chunk in enumerate(chunk_text(document.markdown), start=1):
                record = {
                    "framework": framework,
                    "source_id": document.source_id,
                    "url": document.url,
                    "title": document.title,
                    "chunk_id": f"{safe_name(document.url)}#{index}",
                    "chunk_index": index,
                    "text": chunk,
                }
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")
    return chunks_path


def read_policy_surfaces(framework: str) -> dict[str, str]:
    policy_dir_name = {
        "evergreen-notes": "evergreen",
        "epistemic-sensemaking": "conflicts",
        "structural-proprioception": "proprioception",
        "temporal-scaffolding": "authorship",
    }.get(framework, framework.replace("-", "_"))
    policy_dir = episteme_root() / "policies" / policy_dir_name
    surfaces = {}
    for name in [
        "README.md",
        "validation.sql",
        "diagnostic.toml",
        "authoring_template.toml",
    ]:
        path = policy_dir / name
        if path.exists():
            surfaces[name] = path.read_text()
    return surfaces


def write_prompt_packet(framework: str, chunks_path: Path) -> Path:
    prompt_path = (
        framework_cache_dir(framework) / "prompt_packets" / "extract_evidence.md"
    )
    prompt_path.parent.mkdir(parents=True, exist_ok=True)
    surfaces = read_policy_surfaces(framework)
    surface_summary = "\n\n".join(
        f"## Current policy surface: {name}\n\n```text\n{content[:4000]}\n```"
        for name, content in surfaces.items()
    )
    prompt_path.write_text(
        f"""# Episteme Source Evidence Extraction

Framework: `{framework}`

Read the source chunks in:

```text
{chunks_path}
```

Extract source-grounded evidence records. Return JSON Lines only. Each line must
match this shape:

```json
{{
  "source_id": "string",
  "url": "string",
  "claim": "string",
  "evidence_summary": "string",
  "evidence_quote": "short source quote",
  "principle": "string",
  "policy_candidate": "string",
  "deterministic_candidate": true,
  "target_policy_surface": "validation.sql | diagnostic.toml | authoring_template.toml | review_only",
  "confidence": 0.0
}}
```

Rules:

- Use the fetched source text as evidence.
- Perform the extraction inside the current Codex review session or another
  explicitly reviewed local agent session.
- Keep quotes short.
- Mark subjective writing principles as `deterministic_candidate: false`.
- Mark graph, link, relation-block, or measurable size signals as deterministic
  only when they can be checked through Wendao logical views.
- Do not propose direct file edits.

{surface_summary}
""",
    )
    return prompt_path


def write_report_skeleton(
    framework: str,
    documents: list[FetchedDocument],
    chunks_path: Path,
    prompt_path: Path,
) -> Path:
    report_path = framework_cache_dir(framework) / "alignment" / "report.md"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    rows = "\n".join(
        f"- `{document.source_id}` [{document.title or document.url}]({document.url}) "
        f"markdown_chars={len(document.markdown)} normalized=`{document.normalized_path}`"
        for document in documents
    )
    report_path.write_text(
        f"""# {framework} Source Alignment Report

Status: fetch-and-prompt-packet generated.

## Fetched Sources

{rows}

## Generated Artifacts

- Chunks: `{chunks_path}`
- Codex extraction prompt packet: `{prompt_path}`

## Next Step

Read the chunks and prompt packet in Codex, then draft a reviewable
policy-alignment proposal. This report is a review artifact and does not mutate
policy files.
"""
    )
    return report_path


def run(args: argparse.Namespace) -> int:
    sources_path, entries = load_source_entries(args.framework)
    if args.fetcher != "firecrawl":
        raise SystemExit(f"unsupported fetcher: {args.fetcher}")
    documents = fetch_with_firecrawl(args.framework, entries)
    documents.extend(fetch_local_docs(args.framework, entries))
    chunks_path = write_chunks(args.framework, documents)
    prompt_path = write_prompt_packet(args.framework, chunks_path)
    report_path = write_report_skeleton(
        args.framework,
        documents,
        chunks_path,
        prompt_path,
    )
    print(f"sources={sources_path}")
    print(f"fetched_documents={len(documents)}")
    print(f"chunks={chunks_path}")
    print(f"prompt_packet={prompt_path}")
    print(f"report={report_path}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m wendao_core_lib.episteme_contracts.wendao_episteme_align",
        description="Fetch practice sources and prepare Codex alignment report inputs.",
    )
    parser.add_argument("framework", help="Framework id, for example evergreen-notes")
    parser.add_argument(
        "--fetcher",
        choices=["firecrawl"],
        default="firecrawl",
        help="Fetch provider used for website sources.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
