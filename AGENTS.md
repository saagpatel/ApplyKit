# AGENTS.md (ApplyKit)

<!-- comm-contract:start -->

## Communication Contract

- Inherit global Codex communication and reporting rules from `/Users/d/.codex/AGENTS.override.md` and `/Users/d/.codex/policies/communication/BigPictureReportingV1.md`.
- Repo-specific instructions below add project constraints only; do not restate global voice or status-reporting rules here.
<!-- comm-contract:end -->

Model target: GPT-5.3-Codex High Reasoning

## Hard rules
- Business logic only in crates/applykit_core
- src-tauri RPC only
- UI is render/orchestration only
- Truth Gate prevents invented claims
- Deterministic outputs required (fixtures + snapshots)

## Gates
- cargo test must stay green
- UI lint/build must stay green

## Suggested workstreams
1) applykit_core + CLI + fixtures
2) UI scaffolding + markdown preview
3) Diff viewer + bullet picker
4) LLM adapters + validation

<!-- portfolio-context:start -->
# Portfolio Context

## What This Project Is

deterministic local-first application packet generation

## Current State

Current portfolio truth should be checked in `/Users/d/Projects/GithubRepoAuditor/output/portfolio-truth-latest.json`; recent runs mark this project as `active-product`. This file is only a minimum-viable repo guide, so use the README and release docs before expanding scope.

## Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2021 |
| Desktop shell | Tauri 2 |
| UI | React + TypeScript |
| LLM | Ollama / LM Studio (local) via reqwest |
| Core crates | applykit_core, applykit_llm, applykit_export, applykit_cli |
| Persistence | SQLite via rusqlite |

## How To Run

```bash
# CLI: generate an application packet
cargo run -p applykit_cli -- generate \
  --company "Acme" \
  --role "Senior Engineer" \
  --source "LinkedIn" \
  --baseline 1pg \
  --jd path/to/job_description.txt \
  --outdir ~/applykit_packets

# Desktop app
cargo tauri dev
```

## Known Risks

- This repo only has minimum-viable recovery context today; deeper handoff details may still live in the README and supporting docs.

## Next Recommended Move

Use this context plus the README and supporting docs to resume the next active task, then promote the repo beyond minimum-viable by capturing a dedicated handoff, roadmap, or discovery artifact.

<!-- portfolio-context:end -->
