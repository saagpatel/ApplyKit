
# Testing & Verification
**Date:** 2026-02-14

## Setup and safe checks

Run all commands from the repository root. Use Rust stable, Node.js 22+ and
pnpm 10.20.0 (the `packageManager` in `package.json`). Install workspace
dependencies with `pnpm install --frozen-lockfile`; preserve `Cargo.lock` and
`pnpm-lock.yaml`. The CLI fixtures do not need Ollama. The desktop workspace
member requires [Tauri's platform prerequisites](https://v2.tauri.app/start/prerequisites/);
the canonical desktop CI lane is macOS.

For a focused change, select its package or existing UI test file:

```bash
cargo test --locked -p applykit_cli
pnpm -C ui exec vitest run src/test/responsive.test.tsx
```

For broader local verification:

```bash
cargo test --locked --workspace
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
pnpm -C ui lint
pnpm -C ui test
pnpm -C ui build  # tsc -b + Vite; includes frontend type checking
./.codex/scripts/docs_check.sh
```

For CLI usage, see the [quick start](../README.md#usage). Packet generation
writes outputs and may create `config/signing_key.hex`; it is not a read-only
verification smoke. Run generation in a disposable checkout with synthetic JD,
templates/banks and a temporary output directory, rather than against personal
application data. `./scripts/dev_lean.sh` starts the desktop app and runs cleanup
on exit; use the commands above for verification instead.

For UI changes, run `pnpm -C ui exec playwright install chromium`, then
`CI=1 pnpm -C ui test:e2e:a11y`. The Playwright configuration builds the frontend
and starts its own loopback preview on port 4174; keep that port free. Also
review generate/preview/open-folder behavior in the desktop app using synthetic
fixtures when the native RPC or packet-preview contract changes. Browser tests
use mocked native APIs and do not prove the Tauri integration. Pure docs-only
changes do not need a browser or a running provider.

The full CI gate list is [`.codex/verify.commands`](../.codex/verify.commands),
executed by [CI](../.github/workflows/ci.yml). It additionally requires
`cargo-audit`, `cargo-llvm-cov`, a Tauri 2 CLI and Chromium. Coverage replaces the
repository's `coverage/` output; audit commands access dependency advisory
services. The full gate runner is broader than the safe focused checks above
and includes desktop builds/performance checks. Report unavailable gates and
failures explicitly; do not waive them or infer a pass from a focused test.

## Golden fixtures
fixtures/jd_*.txt + expected snapshots (normalized)

## Test tiers
- Unit: extraction, classification, scoring, truth validation
- Snapshot: full packet outputs (`JD.txt`, `Extracted.json`, `FitScore.md`, `TailorPlan.md`, resumes/messages, `TrackerRow.csv`, `Diff.md`)
- UI smoke: generate + preview + open folder
- UI hook safety: startup data loads are deferred past the initial render, and
  packet tracker form state resets through keyed remounts instead of effect-time
  state synchronization.
- Tauri/UI contract: native packet detail responses serialize to the camelCase
  keys consumed by the packet preview (`packetDetail`, `resume1pg`,
  `hiringManager`, `coverShort`)
- Property-based: JD normalization/extraction determinism and parser robustness
- Export determinism: DOCX/PDF deterministic outputs for identical packet input

## Required Gate Additions (Phase 1)
- Docs check: `./.codex/scripts/docs_check.sh`
- Coverage thresholds: `./.codex/scripts/run_coverage.sh`
- Diff coverage: `node ./.codex/scripts/check_diff_coverage.mjs`
- Required gate waiver enforcement: `node ./.codex/scripts/check_gate_waivers.mjs` (must be empty unless explicitly enabled for emergency renewal)

## Red-team
- prompt injection in JD
- “add skills you don't have” (must be Gap)
- fixture-based injections in `fixtures/jd_redteam_*.txt`
