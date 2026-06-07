# ApplyKit Demo Plan

## Purpose

Use this plan for local, sanitized demos of ApplyKit. The demo should show deterministic packet generation, Truth Gate review, packet export, and tracker status without exposing real resumes, personal contact information, private job records, credentials, or unreviewed LLM output.

## Demo Safety Rules

- Use fixture job descriptions and fixture resume/message templates only.
- Do not use real names, phone numbers, email addresses, employers, job application histories, recruiter notes, or compensation details.
- Keep local LLM output bounded by the Truth Gate; do not present rejected or unreviewed generated claims as final content.
- Do not show `.env` files, local model credentials, private filesystem paths, signing keys, Keychain prompts, or release credentials.
- Treat screenshots in `docs/screenshots/` as UI evidence only, not proof of public release readiness.

## Baseline Scenario

1. Build the React UI from the local workspace.
2. Launch the Tauri desktop shell against sanitized fixture data.
3. Open the New Job flow and paste a fixture job description.
4. Select fixture resume and message banks.
5. Generate a deterministic application packet.
6. Review fit score, tailoring plan, and diff output.
7. Confirm every generated claim traces back to approved source material.
8. Export the packet and verify expected files are present.
9. Update tracker status for the fixture job.
10. Review release-gate status without exposing private local paths or secrets.

## Evidence To Capture

- New Job setup with fixture company, role, source, and sanitized job description.
- Template bank selection using fixture resume and message sources.
- Packet generation result with output path redacted or generic.
- Fit score and gap list.
- Tailor plan and diff view.
- Resume and message review with traceable, non-sensitive claims.
- Export flow for markdown, DOCX, or PDF as applicable.
- Tracker update for the fixture job.
- Release or verification status showing unresolved external gates clearly.

## Verification Notes

Before using this demo externally, run:

```bash
pnpm -C ui build
pnpm -C ui test
cargo test
```

Record anything not verified, especially notarization, updater signing, packaged installer behavior, local model availability, and DOCX/PDF rendering on a clean machine.

## Current Status

No demo capture is committed yet. This folder defines the reusable demo-script surface for future rehearsals and portfolio evidence.
