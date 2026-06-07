# One-Pager Outline

## Product Summary

ApplyKit is a local-first desktop and CLI assistant that turns a job description into a deterministic, truth-gated application packet: tailored resume, cover note, fit score, tailoring plan, diff, export files, and tracker row.

## Audience

- Job seeker preparing fewer, higher-quality applications.
- Career coach or reviewer checking whether packet claims are defensible.
- Reviewer evaluating local-first desktop product quality across Tauri, Rust, React, SQLite, and local LLM surfaces.

## Key Value

- Generates application packets from approved local templates and skill banks.
- Keeps resume material, job descriptions, packet history, and exports local by default.
- Uses a Truth Gate so generated claims must trace back to approved source material.
- Produces deterministic outputs for the same job description and configuration.
- Supports CLI scripting and a desktop review workflow.

## Proof Points

- Tauri 2 desktop shell with React and TypeScript UI.
- Rust workspace split across core, LLM, export, CLI, and Tauri command surfaces.
- SQLite-backed local job and packet tracking via `rusqlite`.
- Local LLM integration through Ollama, LM Studio, or compatible llama.cpp providers.
- Export workflow for packet review and downstream sharing.

## Current Demo Limits

- Public distribution, notarization, updater signing, and packaged installer evidence require separate release validation.
- Real resumes, personal contact data, job histories, and recruiter messages should not be used in public demos.
- Local LLM availability depends on the operator machine and should be stated during rehearsal.
- DOCX and PDF output should be verified on the target demo machine before external use.
