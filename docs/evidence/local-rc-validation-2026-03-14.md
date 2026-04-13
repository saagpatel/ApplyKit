# Local RC Validation - 2026-03-14

## Purpose
Current-date validation artifact for ApplyKit local release-candidate readiness on the root repository checkout.

## What was validated
- UI dependencies installed successfully
- Playwright browser dependency installed successfully
- `cargo tauri dev` launched the native app process on macOS
- Fixture packet generation succeeded and produced canonical packet outputs
- UI lint, unit/integration tests, and Playwright accessibility smoke passed
- Rust audit and UI audit blockers were remediated
- Canonical verify command chain passed end to end
- Desktop package build succeeded and produced `ApplyKit.app`

## Key outcomes
- Local RC status: pass
- Sev-1 unresolved: 0
- Sev-2 unresolved: 0
- Canonical verify: pass
- Desktop debug bundle: pass
- Release-style local packaging helper: available via `./scripts/release/build_macos_release.sh`

## Evidence summary
- Fixture packet output:
  - `/tmp/applykit_packets/Acme_Senior_Support_Engineer_2026-03-14`
- Debug app bundle:
  - `target/debug/bundle/macos/ApplyKit.app`
- Canonical verification command:
  - `bash ./.codex/scripts/run_verify_commands.sh`

## Notable fixes included in this validation
- Upgraded vulnerable Rust lockfile dependency (`quinn-proto`) to clear `cargo audit -D warnings`
- Overrode vulnerable frontend transitive dependency (`flatted`) to clear `pnpm -C ui audit --audit-level high`
- Hardened App workflow handling for generated job id resolution and copy/export error toasts
- Added app-level integration coverage for generate/review/export/tracker flows
- Added stronger native command smoke coverage in `src-tauri` tests

## Remaining non-blocking follow-up
- Apple signing/notarization remains credential-dependent release-environment work
- A deeper native GUI automation harness can still be added later if desired
