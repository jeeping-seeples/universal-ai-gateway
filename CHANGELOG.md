# Changelog

## Unreleased
- Implemented Phase 4: Controlled CLI Adapter.
- Added `CliProvider` with executable allowlists, environment variable clearing (`env_clear`), timeouts, and maximum output truncation.
- Prevented arbitrary shell execution and prohibited unallowlisted binary paths.
- Added streaming and non-streaming responses for CLI-driven models.
- Implemented Phase 3: Streaming, routing, bounded retries, and request IDs.
- Added Server-Sent Events (SSE) streaming support for `POST /v1/chat/completions` with `"stream": true`.
- Added request ID middleware (`x-request-id`) with propagation and structured log attributes without leaking payload secrets.
- Added deterministic model routing with optional fallback model (`gateway.fallback_model`).
- Added bounded retries with exponential backoff on transient errors (HTTP 429, 503, 504).
- Implemented Phase 2: OpenAI-compatible hosted provider adapter.
- Added `OpenAICompatibleProvider` with configurable endpoint (`base_url`), bearer auth, timeout, bounded parsing, and upstream error mapping.
- Added secret masking (`SecretString`) so API keys are never leaked in logs or error messages.
- Added support for loading credentials via TOML with fallback to `OPENAI_API_KEY` and `OPENAI_BASE_URL` env vars.
- Added deterministic mock server integration tests via `wiremock` covering success, bearer auth, error forwarding, and timeouts.
- Implemented Phase 1 skeleton in Rust using Axum and Tokio.
- Added TOML and environment variable configuration loading defaulting to localhost port 9471.
- Implemented structured error model with OpenAI-compatible JSON error responses.
- Implemented `Provider` trait and deterministic `MockProvider` and `ProviderRegistry`.
- Implemented endpoints `GET /health`, `GET /v1/models`, and `POST /v1/chat/completions`.
- Added unit tests and integration tests with 100% test pass rate.
- Added repository overview, agent operating rules, phased plan, decision log, progress tracker, blockers list, and Jules audit brief.
