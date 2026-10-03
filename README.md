# ApplyKit

[![Rust](https://img.shields.io/badge/rust-%23dea584?style=flat-square&logo=rust)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Paste a job description, get a truth-gated application packet — no cloud uploads.

ApplyKit generates truth-gated application packets from job descriptions. The truth gate checks approved bullet provenance, known tools, and prohibited claim patterns; it does not validate every claim semantically. Rule-based tailoring is deterministic with fixed inputs, local templates/banks, config, and run date; optional LLM rewrites and manifest timestamps can vary. Runs entirely locally with optional Ollama or OpenAI-compatible local adapters.

## Features

- **Truth Gate** — checks approved bullet provenance, known tools, and prohibited claim patterns
- **Deterministic tailoring** — fixed inputs, templates/banks, config, and run date with LLM disabled; manifest timestamps still vary
- **Full application packet** — tailored resume(s), cover letter, fit score, tailor plan, diff, and tracker CSV
- **Local LLM** — Ollama, LM Studio, or any llama.cpp-compatible provider
- **CLI + desktop** — `applykit generate` CLI for scripting; Tauri desktop UI for interactive use
- **Modular crate design** — core, LLM adapters, export, and CLI as separate crates

## Quick Start

### Prerequisites
- Rust stable toolchain
- Node.js 22.x at 22.22.2 or later and pnpm 10.20.0 (the locked UI dependencies require this Node 22 patch level).
- Tauri CLI for desktop builds:
  ```bash
  cargo install tauri-cli --version "^2.0" --locked
  ```
- [Ollama](https://ollama.com) for local LLM adapter workflows. The CLI quick start below does not require Ollama; unavailable LLM requests fall back to rule-based generation.

### Installation
```bash
git clone https://github.com/saagpatel/ApplyKit
cd ApplyKit
pnpm -C . install --frozen-lockfile
```

### Usage
```bash
mkdir -p path/to
printf '%s\n' 'Acme is hiring a Senior Engineer to build reliable Rust and TypeScript tooling, own local-first workflows, improve developer experience, and ship deterministic automation for small teams.' > path/to/job_description.txt

# CLI: generate an application packet
cargo run -p applykit_cli -- generate \
  --company "Acme" \
  --role "Senior Engineer" \
  --source "LinkedIn" \
  --baseline 1pg \
  --jd path/to/job_description.txt \
  --outdir ~/applykit_packets

# Desktop app build
cargo tauri build
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2021 |
| Desktop shell | Tauri 2 |
| UI | React + TypeScript |
| Frontend build | Vite with typed stylesheet side-effect imports |
| LLM | Ollama / LM Studio (local) via reqwest |
| Core crates | applykit_core, applykit_llm, applykit_export, applykit_cli |
| Persistence | SQLite via rusqlite |

## License

MIT

## Verification

See [Testing & Verification](docs/testing.md) for prerequisites, focused and
broader checks, fixture safety, and conditional browser/native verification.
