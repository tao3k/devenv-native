---
type: knowledge
metadata:
  title: "Scripts Directory"
---

# Scripts Directory

This directory contains utility scripts for the `xiuxian-artisan-workshop` project.

## Available Scripts

| Script                                                | Purpose                                                                                                  |
| ----------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `benchmark_wendao_gateway_repo_get.py`                | Benchmark live gateway repo GET latency together with bootstrap and repo-index readiness context         |
| `benchmark_wendao_search.py`                          | Benchmark wendao search latency                                                                          |
| `evaluate_wendao_retrieval.py`                        | Evaluate wendao Top1/Top3/Top10 on fixed query matrix                                                    |
| `benchmark_wendao_related.py`                         | Benchmark wendao related latency and PPR diagnostics                                                     |
| `gate_wendao_ppr.sh`                                  | Unified WG2/WG3 gate: retrieval matrix quality + related PPR latency/diagnostics                         |
| `fetch_previous_skills_benchmark_artifact.py`         | Fetch a member file from the latest matching successful GitHub Actions artifact into a local output path |
| `channel/test_xiuxian_daochang_discord_acl_events.py` | Live Discord ingress ACL black-box probe for managed command denial events                               |
| `channel/start-xiuxian-daochang-memory-ci.sh`         | Unified launcher for quick/nightly memory CI gates with latest status/failure aggregation                |
| `channel/memory_ci_finalize.py`                       | Shared artifact finalizer for memory CI launcher (`latest-run`, failure JSON/Markdown)                   |

### Memory CI launchers

```bash
# unified launcher (direct profile selection)
bash scripts/channel/start-xiuxian-daochang-memory-ci.sh --profile quick --foreground

# nightly gate (foreground)
bash scripts/channel/start-xiuxian-daochang-memory-ci.sh --profile nightly --foreground
```

## Running Scripts

All scripts should be run from the project root:

```bash
# Using uv (recommended)
uv run python scripts/script_name.py

# Or directly with python
python scripts/script_name.py
```

## Database Commands

Database operations are now available via the `omni db` CLI command:

```bash
# List all databases
omni db list

# Query knowledge base
omni db query "error handling"

# Show database statistics
omni db stats

# Count records in table
omni db count <table_name>
```
