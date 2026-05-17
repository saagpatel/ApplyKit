# Local RC Smoke Runbook

Use this runbook when validating ApplyKit on a local machine before calling the repo "ready" for a local release candidate.

## Setup
- `pnpm -C ui install --frozen-lockfile`
- `pnpm --dir ui exec playwright install chromium`
- `cargo test`
- `pnpm -C ui build`

## Launch
- `cargo tauri dev`
- Fallback for low-disk environments: `./scripts/dev_lean.sh`

## Core smoke flow
1. Confirm the app window opens to the dashboard and the sidebar can navigate to Jobs, New Job, Review, Banks, Templates, and Settings.
2. Generate a packet from `fixtures/jd_support_ops_01.txt`.
3. Confirm Job Review shows fit score, extracted signals, resume preview, messages, diff, and tracker state.
4. Run Export Markdown, Export DOCX, and Export PDF from the Export tab.
5. Update tracker status, next action, and notes; reopen the job and confirm the updated values persist.
6. Save settings with safe local defaults and confirm generation still works with deterministic fallback.
7. Perform one valid and one invalid save path in Banks and Templates to confirm both success and validation feedback.

## Minimum pass bar
- No setup or launch blocker.
- No generate/review/export/tracker blocker.
- No unhandled copy/export/settings errors in the UI.
- `bash ./.codex/scripts/run_verify_commands.sh` passes.
- `cargo tauri build --debug --bundles app` passes.

## Notes
- Browser-only checks are useful, but the smoke pass should include a real Tauri launch because `ui/src/lib/tauri.ts` requires the Tauri runtime for invoke-backed flows.
- Historical prepared-early release documents remain in `docs/`; use them as planning/evidence references, not as the sole source of current readiness status.
