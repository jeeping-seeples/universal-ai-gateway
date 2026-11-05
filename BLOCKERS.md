# Blockers and open questions

Resolved for Phase 1:
- First hosted provider: OpenAI-compatible interface selected.
- Target device / profile: Raspberry Pi 4B.
- Network binding: `127.0.0.1:9471` (localhost only default).
- Configuration format: TOML with env overrides.

Open questions for subsequent phases:
- Which streaming protocol best fits the client (Server-Sent Events / SSE recommended for Phase 3)?
- What is the intended CLI integration target, and what permissions should it have (Phase 4)?
