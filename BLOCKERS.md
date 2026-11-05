# Blockers and open questions

Resolved through Phase 3:
- First hosted provider: OpenAI-compatible interface implemented.
- Target device / profile: Raspberry Pi 4B.
- Network binding: `127.0.0.1:9471` (localhost only default).
- Configuration format: TOML with env overrides.
- Streaming protocol: Server-Sent Events (SSE).
- Routing & retries: Exact model routing with fallback model support; bounded retries with backoff.

Open questions for subsequent phases:
- Phase 4: Controlled CLI adapter design, allowlisted executables, and human approval boundary.
- Phase 5: ZipChat client integration contract freezing.
