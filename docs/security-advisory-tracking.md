# Security Advisory Tracking

Source of truth for active `cargo audit` ignore entries in `/Users/d/Projects/ApplyKit/.cargo/audit.toml`.

## Current Advisory Matrix (2026-04-12)

| Advisory Group | IDs | Status | Last Validated On | Removal Blocker | Owner | Mitigation Issue | Target Removal Date | Next Review Date |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| GTK3 transitive chain | `RUSTSEC-2024-0411` .. `RUSTSEC-2024-0420`, `RUSTSEC-2024-0429` | residual accepted (issue closed) | 2026-03-01 | Transitively required by `tauri`/`wry` Linux GTK3 stack; no compatible in-range update in dry-run | applykit-platform | AK-301 ([#7](https://github.com/saagar210/ApplyKit/issues/7)) | 2026-03-31 | on next compatible dependency release |
| Macro dependency in GTK3 chain | `RUSTSEC-2024-0370` | residual accepted (issue closed) | 2026-03-01 | Pulled through `glib-macros` in the same GTK3 chain; no direct replacement without upstream stack move | applykit-platform | AK-301 ([#7](https://github.com/saagar210/ApplyKit/issues/7)) | 2026-03-31 | on next compatible dependency release |
| Hash crate transitive warning | `RUSTSEC-2025-0057` | residual accepted (issue closed) | 2026-03-01 | Pulled through `selectors -> kuchikiki -> tauri-utils`; no compatible dry-run update removed chain | applykit-platform | AK-302 ([#8](https://github.com/saagar210/ApplyKit/issues/8)) | 2026-03-31 | on next compatible dependency release |
| Unicode/urlpattern transitive warnings | `RUSTSEC-2025-0075`, `RUSTSEC-2025-0080`, `RUSTSEC-2025-0081`, `RUSTSEC-2025-0098`, `RUSTSEC-2025-0100` | residual accepted (issue closed) | 2026-03-01 | Pulled through `urlpattern -> tauri-utils`; no compatible dry-run update removed chain | applykit-platform | AK-303 ([#9](https://github.com/saagar210/ApplyKit/issues/9)) | 2026-03-31 | on next compatible dependency release |
| Rand transitive warning | `RUSTSEC-2026-0097` | residual accepted (tracked) | 2026-04-12 | Still present after latest compatible `tauri`, `tauri-build`, `tauri-utils`, and lockfile refreshes; active through transitive `reqwest/proptest` plus `tauri-utils`/`wry` chains with no compatible repo-local removal | applykit-platform | AK-305 | 2026-04-30 | on next compatible dependency release |

## 2026-04-12 Revalidation Snapshot

- Canonical strict audit (with tracked ignore list):
  - `cargo audit -D warnings`
  - Result: pass after adding `RUSTSEC-2026-0097` to the tracked ignore list.
- Baseline no-ignore scan:
  - `cd /tmp && cargo audit -f /Users/d/Projects/ApplyKit/Cargo.lock -D warnings`
  - Result: expected non-zero with `RUSTSEC-2026-0097` still active through transitive `reqwest/proptest` and `tauri-utils`/`wry` dependency chains.
- Compatible upgrade checks executed in this cycle:
  - `cargo update -p tauri -p tauri-build -p tauri-utils`
  - `pnpm install --frozen-lockfile=false`
- Outcome:
  - UI high-severity advisories cleared by moving to `vite@7.3.2` plus root workspace `picomatch` overrides.
  - Rust `rand` advisory remains transitive after latest compatible upgrades, so it is tracked as residual accepted risk pending upstream dependency movement.

## Week 3 Baseline Evidence

- Strict baseline command (without local ignore config):
  - `cd /tmp && cargo audit -f <path-to-repo>/Cargo.lock -D warnings --json > <path-to-repo>/docs/evidence/week3-baseline-audit.json`
- Active advisory IDs discovered: 18
- Ignore entries in `.cargo/audit.toml`: 18
- Diff result:
  - stale ignore entries: none
  - missing ignore entries for active advisories: none

## Dependency Path Snapshots (Week 3)

- `fxhash` advisory chain (`RUSTSEC-2025-0057`):
  - `fxhash -> selectors -> kuchikiki -> tauri-utils -> tauri`
- Unicode advisory chain (`RUSTSEC-2025-0075`, `0080`, `0081`, `0098`, `0100`):
  - `unic-ucd-ident -> urlpattern -> tauri-utils -> tauri`
- GTK3 advisory chain:
  - `gtk/glib stack -> wry/tauri-runtime-wry -> tauri`
- `proc-macro-error` advisory (`RUSTSEC-2024-0370`):
  - `proc-macro-error -> glib-macros/gtk3-macros -> gtk/glib stack`

## Feasibility Checks Performed

- `cargo update --workspace --dry-run`
- `cargo update -p tauri-utils --dry-run`
- `cargo update -p glib --dry-run`
- `cargo update -p proc-macro-error --dry-run`

All dry-runs reported no compatible lockfile upgrades that remove the active advisory set.

## 2026-02-28 Revalidation Snapshot

- Baseline command (no local ignore config):
  - `cd /tmp && cargo audit -f <path-to-repo>/Cargo.lock -D warnings --json > <path-to-repo>/docs/evidence/operational-revalidation-2026-02-28-baseline-audit.json`
- Lockfile compatibility update attempt:
  - `cargo update -w`
- Result:
  - active advisory IDs in warnings: 18
  - stale ignore IDs in `/Users/d/Projects/ApplyKit/.cargo/audit.toml`: none
  - compatible lockfile upgrades available to remove advisories: none

## Week 3 Conclusion

- No stale ignore IDs were found, so no ignore removals were applied in this cycle.
- Residual advisory risk remains explicitly tracked and time-bound by owner and mitigation issue.

## 2026-02-28 Operational Follow-up Revalidation

Commands rerun in this cycle:
- Baseline no-ignore scan (source: `docs/week3-checklist.md`):
  - `cd /tmp && cargo audit -f <path-to-repo>/Cargo.lock -D warnings --json > <path-to-repo>/docs/evidence/operational-revalidation-2026-02-28-baseline-audit.json`
  - Result: expected non-zero exit (`EXIT:1`) with 18 informational advisories.
- Canonical strict audit (source: `.codex/verify.commands`):
  - `cargo audit -D warnings`
  - Result: pass (`EXIT:0`).

Comparison results:
- Active advisory IDs from baseline: 18
- Ignore IDs in `.cargo/audit.toml`: 18
- Stale ignore IDs: 0
- Missing ignore IDs for active advisories: 0
- Durable evidence artifact:
  - `docs/evidence/operational-revalidation-2026-02-28.md`

Current posture:
- All tracked advisory groups remain present in baseline scans and owned by `applykit-platform`.
- Mitigation issues are now closed with explicit residual-risk acceptance:
  - AK-301: [#7](https://github.com/saagar210/ApplyKit/issues/7) (closed 2026-03-01)
  - AK-302: [#8](https://github.com/saagar210/ApplyKit/issues/8) (closed 2026-03-01)
  - AK-303: [#9](https://github.com/saagar210/ApplyKit/issues/9) (closed 2026-03-01)
  - Program closeout: AK-304 [#12](https://github.com/saagar210/ApplyKit/issues/12) (closed 2026-03-01)

## 2026-03-01 Closure Decision

- Final closure execution completed under AK-304 decision rules.
- Final evidence:
  - `cargo audit -D warnings`: pass
  - baseline no-ignore scan: expected non-zero (18 active advisory IDs)
  - `.cargo/audit.toml` ignores: 18
  - stale ignore IDs: 0
  - missing ignore IDs: 0
  - `cargo update -w --dry-run`: no compatible lockfile upgrades
- Reopen criteria:
  - reopen AK-301/302/303 if a compatible dependency path removes the affected advisory IDs while preserving green canonical verify.

## Phase 4 Security Revalidation (Prepared Early)

- Canonical strict audit (with tracked ignore list): pass
  - `/tmp/applykit_phase4_week5_day3_cargo_audit.log`
  - `/tmp/applykit_phase4_week5_day5_cargo_audit.log`
- Baseline no-ignore advisory scan: expected fail (informational warnings enabled)
  - `/tmp/applykit_phase4_week5_day5_baseline_audit.log`
  - `/tmp/applykit_phase4_week5_day5_baseline_audit_summary.log`
- Revalidation outcome:
  - active advisory IDs: 18
  - ignored advisory IDs in `.cargo/audit.toml`: 18
  - stale ignore IDs: none

## 2026-03-01 Hardening Cycle Revalidation

- Baseline command (no local ignore config):
  - `cd /tmp && cargo audit -f /Users/d/Projects/ApplyKit/Cargo.lock -D warnings --json > /tmp/applykit_security_baseline_2026-03-01.json`
  - Result: expected non-zero (`exit 1`) with informational warnings enabled.
- Canonical strict audit:
  - `cargo audit -D warnings`
  - Result: pass.
- Advisory drift check:
  - active advisory IDs from baseline: 18
  - ignore IDs in `.cargo/audit.toml`: 18
  - stale ignore IDs: none
  - missing ignore IDs: none
