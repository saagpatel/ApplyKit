# macOS Release Distribution

This document covers the practical release-distribution path for ApplyKit on macOS after local RC readiness is complete.

## What is validated in-repo
- Local development launch via `cargo tauri dev`
- Canonical repo verification via `bash ./.codex/scripts/run_verify_commands.sh`
- Desktop packaging in debug mode via `cargo tauri build --debug --bundles app`
- Local release-style artifact build via `./scripts/release/build_macos_release.sh`

## What still depends on external credentials
- Apple signing identity selection
- Apple notarization credentials and submission
- Final release artifact publishing/distribution

## Recommended operator flow
1. Run `bash ./.codex/scripts/run_verify_commands.sh`
2. Run `./scripts/release/build_macos_release.sh`
3. Confirm release artifacts exist in `target/release/bundle/`
4. If signing credentials are available, perform signing/notarization in the release environment
5. Record final artifact paths, checksums, operator, and decision in the release runbook/evidence doc

## Notes
- The repository can prove local package readiness without Apple credentials.
- Treat signing/notarization as release-environment work, not as a blocker for local RC validation on a developer machine.
