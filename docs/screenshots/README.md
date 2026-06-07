# Screenshot Capture Plan

## Purpose

This folder tracks screenshots needed for portfolio review, release notes, and demo rehearsal. Captures must use sanitized fixture data only.

## Capture Matrix

| File                       | Surface        | Caption                                                           |
| -------------------------- | -------------- | ----------------------------------------------------------------- |
| `01-new-job.png`           | New Job        | Fixture job description entered with sanitized company and role.  |
| `02-template-banks.png`    | Template banks | Fixture resume and message banks selected for packet generation.  |
| `03-generation-result.png` | Packet output  | Deterministic packet generated with private paths hidden.         |
| `04-fit-score.png`         | Fit score      | Fit score and gaps calculated from fixture-only inputs.           |
| `05-tailor-plan.png`       | Tailor plan    | Proposed resume edits constrained by approved source material.    |
| `06-diff-view.png`         | Diff review    | Resume or message diff showing sanitized generated changes.       |
| `07-truth-gate.png`        | Truth Gate     | Claims traced to approved resume or skill bank sources.           |
| `08-export.png`            | Export         | Markdown, DOCX, or PDF export prepared from the fixture packet.   |
| `09-tracker.png`           | Tracker        | Fixture job status updated after packet review.                   |
| `10-release-readiness.png` | Release gates  | Dry-run or readiness checks with unresolved external gates shown. |

## Capture Rules

- Use fixture resumes, fixture job descriptions, and generic company names.
- Hide or replace local filesystem paths unless they are intentionally generic.
- Do not show real personal contact information, employers, job history, recruiter messages, compensation notes, API keys, model credentials, or signing material.
- Include a short caption beside each final screenshot in release or portfolio materials.
- Re-capture screenshots after visible UI changes, packet contract changes, export changes, or Truth Gate behavior changes.

## Current Status

No screenshots are committed yet. This plan is the source of truth for the first capture pass.
