# Verified Application Packet (VAP) Manifest — `vap/1`

The VAP manifest is the machine-readable contract between ApplyKit (which
generates a truth-gated application packet) and any downstream consumer (e.g.
JobCommandCenter, which tracks the application and prepares ATS fields for a
human to review and submit).

ApplyKit writes one `packet.manifest.json` into every generated packet
directory, alongside the packet artifacts. A consumer reads the manifest to:

1. **Identify** the packet (`packet_id`, content-addressed and deterministic).
2. **Trust** the truth-gate verdict (`truth`).
3. **Detect tampering / staleness** by re-hashing each listed artifact and
   comparing against `artifacts[].sha256`.
4. **Verify provenance** (Phase 1b) via the optional Ed25519 `signature`.

This is a **data contract**, not shared code. Both sides pin the
`schema_version` string. An unknown or newer `schema_version` MUST be refused,
never best-effort parsed.

## Design invariants

- **`packet_id` is application identity, not a content fingerprint.** It is
  `sha256:` + SHA-256 over `"<company>\0<role>\0<jd_sha256>"`. Re-tailoring the
  same role against the same JD yields the same `packet_id` — a new *version* of
  the same application, not a new application. This mirrors the identity ApplyKit
  keys job records on, so both sides agree on what "the same application" means.
- **Identity is separate from integrity.** `packet_id` answers *which application
  is this*; `artifacts[].sha256` answers *have the bytes changed*. Same
  `packet_id` + changed artifact hashes = a revised packet for a job already
  known (version bump). This is deliberately independent of output location and
  `generated_at`, both of which would otherwise break stable identity.
- **The manifest never hashes itself or `ReviewData.json`.** The artifact set is
  exactly the packet deliverables (`files_written`); the manifest and
  `ReviewData.json` are metadata layers *about* the packet and are excluded by
  construction.
- **Truth status is scoped to packet origin.** A consumer must not represent
  fields it fills itself (heuristics, AI fallback, user edits) as covered by
  this manifest's `truth` verdict.

## Consumer truth-status model (recommended)

| Consumer status | Condition |
|---|---|
| `VERIFIED` | schema supported · (Phase 1b) signature valid · every `artifacts[].sha256` matches the file on disk · `truth.passed == true` |
| `STALE` | schema supported · a listed artifact's on-disk hash no longer matches (edited after generation) → re-run ApplyKit to restore |
| `UNVERIFIED` | no manifest · unsupported `schema_version` · (Phase 1b) missing/invalid signature |

## Shape

```jsonc
{
  "schema_version": "vap/1",
  "packet_id": "sha256:<hex>",           // application identity: sha256(company\0role\0jd_sha256)
  "generator": {
    "name": "applykit",
    "version": "0.1.0",                   // CARGO_PKG_VERSION
    "git_sha": "abc1234"                  // optional; present if APPLYKIT_GIT_SHA set at build
  },
  "generated_at": "2026-07-09T12:34:56-07:00",  // RFC 3339; NOT part of packet_id
  "source": {
    "company": "Acme",
    "role": "Senior Engineer",
    "jd_sha256": "<hex>",                 // sha256 of the raw input JD (matches ApplyKit's job id)
    "source_platform": "LinkedIn"
  },
  "truth": {
    "method": "deterministic-rule-gate/v1",   // honest label — NOT adversarial
    "gate_version": "1",
    "passed": true,
    "provenance_complete": true,
    "violations": [],
    "unknown_tools": [],
    "claim_issues": []
  },
  "fit": {
    "track": "Automation/AI Ops",
    "total": 78,
    "gaps": []
  },
  "artifacts": [
    { "role": "resume_1pg", "path": "Resume_1pg_Tailored.md", "sha256": "<hex>", "format": "md" },
    { "role": "cover_note", "path": "CoverNote_Short.md",     "sha256": "<hex>", "format": "md" }
    // ... one entry per packet deliverable, sorted by path
  ],
  "signature": {                          // OPTIONAL — absent on unsigned packets
    "alg": "ed25519",
    "public_key": "<hex of 32-byte Ed25519 public key>",
    "public_key_id": "<sha256 fingerprint of public_key>",
    "signature": "<hex of 64-byte signature over the signing payload below>"
  }
}
```

## Signing payload (what the signature covers)

The Ed25519 signature is computed over a deterministic, formatting-independent
payload — NOT the raw JSON — so an independent verifier reconstructs the exact
signed bytes without agreeing on JSON serialization. The payload is the UTF-8
bytes of these newline-joined lines:

```
schema=<schema_version>
packet_id=<packet_id>
artifact\0<path>\0<sha256>      // one per artifact, each line sorted ascending
truth_passed=<true|false>
```

Editing any artifact (its hash), the identity (its id), the schema, or the truth
verdict changes this payload and invalidates the signature. `generated_at` and
`generator` are intentionally NOT signed (non-security-relevant metadata).

Verification: rebuild the payload from the parsed manifest, decode
`signature.public_key`, and verify. A valid signature proves **integrity**. To
also assert **provenance**, pin the expected `public_key_id` in the consumer and
compare. ApplyKit persists its keypair at `config/signing_key.hex` (0600,
gitignored) and embeds the public key + fingerprint in every signed manifest.

### Artifact roles

`jd`, `extracted`, `fit_score`, `tailor_plan`, `resume_1pg`, `resume_2pg`,
`recruiter_message`, `hiring_manager_message`, `cover_note`, `diff`,
`tracker_csv`, `meta`, `other`.

## Versioning

`schema_version` is a hard gate. `vap/1` is the current version. A consumer that
does not recognize the version refuses the packet with a clear message rather
than parsing partially.

## Phase status

- **Phase 1a:** manifest emitted with application-identity `packet_id` and
  per-artifact SHA-256.
- **Phase 1b (current):** ApplyKit signs every manifest with a per-install
  Ed25519 keypair (`config/signing_key.hex`, 0600). The `signature` block embeds
  the public key + fingerprint. A consumer verifies integrity against the
  embedded key and asserts provenance by pinning `public_key_id`.
