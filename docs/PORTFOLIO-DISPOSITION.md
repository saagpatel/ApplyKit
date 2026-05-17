# ApplyKit — Portfolio Disposition

**Status:** Active (Tauri 2 + Rust workspace, dual CLI + desktop
distribution, local-LLM-dependent) — Cargo workspace with
**modular crate design** (core, LLM adapters, export, CLI) plus
Tauri 2 desktop UI on `origin/main`. Generates **deterministic
truth-gated** job-application packets using local LLM (Ollama /
LM Studio / llama.cpp). Recent commits show CI / perf / security
hardening (`rustls-webpki` lockfile bump, build-time jitter
tolerance, pnpm package manager version pin), but **no v1.0
release closeout cadence yet**. **Joins signing cluster as
candidate #32, but Active state.** Introduces new sub-pattern:
**dual-distribution Tauri** (CLI + Desktop in one Cargo workspace).

> Disposition uses strict `origin/main` verification.
> **New sub-pattern: dual CLI + desktop distribution** from one
> Cargo workspace.

---

## Verification posture

Only `origin` (`saagpatel/ApplyKit`). Clean.

`origin/main`:

- Tip: `2398d29` fix(perf): tolerate small build-time jitter
- Recent infrastructure / security commits:
  - `2398d29` fix(perf): tolerate small build-time jitter
  - `5a092ca` fix(security): bump rustls webpki lockfile
  - `ee300d1` fix(ci): use package manager pnpm version
  - `f2b1ce6` chore: normalize PR template filename to uppercase
  - `c354520` chore(codex): update pipeline source, docs,
    lockfiles, and perf baselines
- Full OSS scaffolding wave further back
- Repo tree (`origin/main`):
  - `Cargo.toml` + `Cargo.lock` + `.cargo/` (Rust workspace)
  - `src-tauri/` (Tauri 2 desktop)
  - `package.json` (frontend deps)
- **No v1.0.0 version bump** visible
- **No .dmg distribution build deps** commit visible
- Default branch: `main`

---

## Current state in one paragraph

ApplyKit generates **deterministic truth-gated** job-application
packets from job descriptions using local LLMs. Two distribution
surfaces:

1. **CLI** (`applykit_cli` crate): `cargo run -p applykit_cli --
   generate --company X --role Y --jd ./jd.txt` — scriptable
2. **Desktop** (`src-tauri/`): Tauri 2 interactive UI for the same
   workflow

The **Truth Gate** ensures generated claims trace back to approved
local resume templates and skill banks — no fabrication. **Same
JD + same config = same output** (deterministic). Local LLM
backend (Ollama / LM Studio / llama.cpp). Output: tailored
resume(s) + cover letter + fit score + tailor plan + diff +
tracker CSV.

Active state because: no v1.0 release closeout cadence (no version
bump, no .dmg deps, no CSP commit), recent commits are
infrastructure work, and dual-distribution (CLI + Desktop) needs
operator decision about which surface is primary.

For full detail see `README.md` on `origin/main`.

---

## Why "Active" (not Release Frozen) — distinct from cluster siblings

Standard Tauri 2 cluster members have explicit v1.0 release
closeout cadence (CSP + version bump + Cargo.lock for v1.0.0 +
.dmg build deps + baseline tests). ApplyKit has none of these on
canonical main yet:

| Signal | Cluster RF members | **ApplyKit** |
|---|---|---|
| v1.0.0 version bump | All ✓ | **Missing** |
| Cargo.lock for v1.0.0 | All ✓ | **Missing** (Cargo.lock exists but not tagged) |
| .dmg distribution build deps | All ✓ | **Missing** |
| CSP commit | All ✓ | **Missing** (may be configured but not in a tagged commit) |
| Baseline tests | Most ✓ | unverified |
| Substantive features | All ✓ | ✓ (truth gate + deterministic generation + dual surfaces) |
| Distribution model | Single (DMG) | **Dual: CLI + Desktop** |

The right transition is **Active → Release Frozen** once:
1. Operator decides primary distribution surface (CLI ships via
   `cargo install applykit_cli` and/or PyPI-equivalent for Rust;
   Desktop ships via signed DMG)
2. v1.0 closeout cadence applied to whichever surfaces ship

---

## New sub-pattern: dual CLI + desktop from one Cargo workspace

ApplyKit is the first portfolio app shipping **both** a CLI and a
desktop UI from a single Cargo workspace. This is distinct from:

- Pure CLI (mcpforge, MCPAudit, GithubRepoAuditor): no GUI surface
- Pure desktop (NetworkDecoder, IRS, etc.): CLI is operator-only,
  not user-facing
- Hybrid signing+extension (APIReverse): desktop + browser
  extension, but both UI surfaces
- Tauri + Python sidecar (JobMarketHeatmap): single desktop UI
  with bundled service backend

ApplyKit's dual-distribution lets users choose:
- **CLI users**: scripters, CI automation, fast batch packet
  generation
- **Desktop users**: interactive workflow, visual diff, template
  management

Operator must decide which surface is primary (or commit to
shipping both equally). Pattern other dual-distribution Rust
workspaces should follow.

---

## Cluster taxonomy update

| Cluster | Count | Notes |
|---|---|---|
| **Signing (Apple desktop)** | **32** (provisional — Active state) | 30 RF + 1 IRS-local-pending + 1 JobMarketHeatmap-Active + **1 ApplyKit-Active** |

The signing cluster now has 2 Active members (JobMarketHeatmap +
ApplyKit) alongside 30 Release Frozen + 1 with local-pending
work. The cluster's lattice has matured: cluster × state are
independent axes (matching iOS App Store + PyPI cluster pattern).

---

## Unblock trigger (operator)

1. **Decide distribution surfaces**:
   - **CLI primary**: `cargo install applykit_cli` (publishes to
     crates.io) + optional Homebrew formula + standalone desktop
     download.
   - **Desktop primary**: DMG via Apple Developer ID + CLI as
     "advanced users" feature.
   - **Both equal**: parallel release cadence, separate version
     numbers per surface.
2. **Apply Tauri 2 v1.0 release closeout cadence** (CSP +
   baseline tests + version bump + Cargo.lock + .dmg deps) to
   desktop surface.
3. **Apply CLI release cadence** if shipping to crates.io: `cargo
   publish` from `applykit_cli` crate + version tag.
4. **Local LLM onboarding UX** — same concern as thought-trails:
   Ollama / LM Studio install + model pull guidance.
5. **Truth Gate documentation** — strong marketing hook ("no
   hallucinated claims"). Worth elaborating in README + Chrome
   Web Store-style listing.
6. **Modular crate design** — confirm crate boundaries are clean
   (core, LLM adapters, export, CLI). If well-factored, the LLM
   adapters could become their own published crate.

Estimated operator time: ~6-8 hours (dual-surface release
coordination is the dominant cost).

---

## Portfolio operating system instructions

| Aspect | Posture |
|---|---|
| Portfolio status | `Active (Tauri 2 + Rust dual-distribution prep arc)` |
| Distribution model | **Dual**: CLI (crates.io? Homebrew?) + Desktop (DMG via Apple Developer ID) |
| Review cadence | Active — driven by distribution-surface decision + release closeout |
| Resurface conditions | (a) Distribution-surface decision, (b) release closeout cadence applied, (c) local LLM API stability, (d) v1.1 scope, (e) Truth Gate elaboration |
| Co-batch with | Signing cluster — **now 32 repos** (30 RF + 1 IRS-pending + 2 Active) |
| Sub-pattern | **Dual CLI + Desktop from one Cargo workspace** (new) |
| Sub-pattern (shared) | **Local-LLM-dependent** (same as thought-trails — Ollama / LM Studio / llama.cpp prerequisite) |
| Special concern | **Distribution-surface decision is operator-judgment-driven.** CLI primary vs desktop primary materially changes packaging strategy. |
| Special concern | **Local LLM onboarding friction** — same as thought-trails. |
| Special concern | **Truth Gate is the differentiating feature** — lead positioning with it. |

---

## Reactivation procedure

1. Verify branch tracking.
2. Review stash `r17-applykit-stash` (deleted PR_TEMPLATE.md +
   modified AGENTS.md). The PR template deletion is interesting —
   operator may be normalizing to uppercase per `f2b1ce6` commit.
3. **Decide distribution surface(s)** as priority-1 product
   decision before further development.
4. Apply Tauri 2 v1.0 closeout cadence to chosen surface(s).
5. Run `cargo test` across all crates.
6. Test CLI on a real job description; test desktop UI flow.
7. Verify local LLM integration with at least Ollama
   (`llama3.1:8b` is a reasonable starter model).

---

## Last known reference

| Field | Value |
|---|---|
| `origin/main` tip | `2398d29` fix(perf): tolerate small build-time jitter |
| Default branch | `main` |
| Build system | **Cargo workspace** (core + LLM adapters + export + CLI crates) + Tauri 2 desktop UI + pnpm frontend deps |
| Distribution model | **Dual**: CLI + Desktop (operator decision pending) |
| Required runtime | **Local LLM** (Ollama / LM Studio / llama.cpp) |
| Distinguishing tech | **Truth Gate** (no hallucinated claims) + **deterministic output** (same input + same config = same packet) + **dual distribution** + modular crate design |
| Phases shipped | Architectural backbone (core + LLM adapters + export + CLI + desktop UI). **No v1.0 release closeout cadence yet.** |
| Migration state | No `legacy-origin` remote |
| Distinguishing feature | **32nd signing cluster candidate (Active state). Introduces dual CLI + Desktop distribution sub-pattern.** Second Tauri 2 Active member (after JobMarketHeatmap) — confirms Active state is real in signing cluster, not anomaly. |
