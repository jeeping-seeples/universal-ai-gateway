# Blockers and open questions

Resolved through Phase 4:
- First hosted provider: OpenAI-compatible interface implemented.
- Target device / profile: Raspberry Pi 4B.
- Network binding: `127.0.0.1:9471` (localhost only default).
- Configuration format: TOML with env overrides.
- Streaming protocol: Server-Sent Events (SSE).
- Routing & retries: Exact model routing with fallback model support; bounded retries with backoff.
- Controlled CLI adapter: Strict executable allowlist, environment clearing, bounded output, execution timeout, disabled by default.

Open questions for subsequent phases:
- Phase 5: ZipChat client integration contract freezing and client compatibility testing.
