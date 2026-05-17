# ApplyKit

[![Rust](https://img.shields.io/badge/rust-%23dea584?style=flat-square&logo=rust)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Paste a job description, get a complete application packet — no hallucinated claims, no cloud uploads.

ApplyKit generates deterministic, truth-gated application packets from job descriptions. A truth gate ensures every generated claim traces back to your approved local resume templates and skill banks — no fabrication. The same JD + same config always produces the same output. Runs entirely locally via Ollama.

## Features

- **Truth Gate** — generated claims come only from approved local templates and skill banks
- **Deterministic output** — same input + same config = same packet every time
- **Full application packet** — tailored resume(s), cover letter, fit score, tailor plan, diff, and tracker CSV
- **Local LLM** — Ollama, LM Studio, or any llama.cpp-compatible provider
- **CLI + desktop** — `applykit generate` CLI for scripting; Tauri desktop UI for interactive use
- **Modular crate design** — core, LLM adapters, export, and CLI as separate crates

## Quick Start

### Prerequisites
- Rust stable toolchain
- Node.js 22+ and pnpm
- [Ollama](https://ollama.com) running locally

### Installation
```bash
git clone https://github.com/saagpatel/ApplyKit
cd ApplyKit
pnpm -C ui install
```

### Usage
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
pnpm tauri dev
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2021 |
| Desktop shell | Tauri 2 |
| UI | React + TypeScript |
| LLM | Ollama / LM Studio (local) via reqwest |
| Core crates | applykit_core, applykit_llm, applykit_export, applykit_cli |
| Persistence | SQLite via rusqlite |

## License

MIT
