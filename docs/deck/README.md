# Demo Deck Outline

## Purpose

Use this outline to build a short demo deck after the screenshot capture pass. The deck should support a live walkthrough, not replace it.

## Suggested Slides

1. **Problem**: High-quality applications take too long, and generic AI drafts can invent unsupported claims.
2. **Product**: ApplyKit generates deterministic application packets from local, approved source material.
3. **Local-First Architecture**: Tauri 2 desktop shell, Rust core, React UI, SQLite storage, local LLM provider, and CLI.
4. **Packet Workflow**: Paste a job description, select source banks, generate packet, and review outputs.
5. **Truth Gate**: Every generated claim must trace back to approved resume templates or skill banks.
6. **Review Workflow**: Fit score, gaps, tailor plan, diff, resume, and message surfaces.
7. **Export Workflow**: Markdown, DOCX, PDF, and tracker artifacts prepared from fixture data.
8. **Security Posture**: No cloud upload requirement, sanitized demos, local storage, and explicit release-gate evidence.
9. **Verification**: Rust tests, UI tests, build checks, export checks, and unresolved external release gates.
10. **Next Steps**: Screenshot capture, one-pager rendering, deck build, and release evidence refresh.

## Rehearsal Notes

- Keep the live demo on fixture job descriptions and fixture resume banks.
- State whether the LLM provider is mocked, local, or unavailable before generation starts.
- Do not open real resumes, private job trackers, recruiter messages, `.env` files, local model credentials, signing keys, or Keychain prompts.
- Keep a fallback path: screenshots can carry the story if the desktop shell, local model, or export renderer is unavailable.
